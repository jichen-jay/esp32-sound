/*
 * SPDX-FileCopyrightText: 2021-2024 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Unlicense OR CC0-1.0
 */

/* I2S Digital Microphone Recording Example */
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <sys/unistd.h>
#include <sys/stat.h>
#include "sdkconfig.h"
#include "esp_log.h"
#include "esp_err.h"
#include "esp_system.h"
#include "esp_vfs_fat.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "driver/i2s_pdm.h"
#include "driver/gpio.h"
#include "driver/spi_common.h"
#include "sdmmc_cmd.h"
#include "format_wav.h"
#include "esp_log.h"

static const char *TAG = "pdm_rec_example";

// --- Fixed pin definitions ---
#define I2S_MIC_CLK_GPIO 14       // Your SCLK pin
#define I2S_MIC_DATA_GPIO 33      // Your DOUT pin
#define I2S_MIC_SAMPLE_RATE 16000 // Corrected sample rate
#define RECORDING_TIME_SEC 10     // Record for 10 seconds

// SD Card SPI pins (using working configuration from emmc_example)
#define PIN_NUM_MISO 22 // Working pin from emmc_example
#define PIN_NUM_MOSI 19 // Working pin from emmc_example
#define PIN_NUM_CLK 21  // Working pin from emmc_example
#define PIN_NUM_CS 0    // Working pin from emmc_example

#define SPI_DMA_CHAN SPI_DMA_CH_AUTO
#define NUM_CHANNELS (1) // For mono recording only!
#define SD_MOUNT_POINT "/sdcard"
#define SAMPLE_SIZE (CONFIG_EXAMPLE_BIT_SAMPLE * 1024)
#define BYTE_RATE (I2S_MIC_SAMPLE_RATE * (CONFIG_EXAMPLE_BIT_SAMPLE / 8)) * NUM_CHANNELS

// When testing SD and SPI modes, keep in mind that once the card has been
// initialized in SPI mode, it can not be reinitialized in SD mode without
// toggling power to the card.
sdmmc_card_t *card;
i2s_chan_handle_t rx_handle = NULL;

static int16_t i2s_readraw_buff[SAMPLE_SIZE];
size_t bytes_read;
const int WAVE_HEADER_SIZE = 44;

void mount_sdcard(void)
{
    esp_err_t ret;
    esp_vfs_fat_sdmmc_mount_config_t mount_config = {
        .format_if_mount_failed = true,
        .max_files = 5,
        .allocation_unit_size = 16 * 1024 // Use larger allocation unit like emmc_example
    };
    ESP_LOGI(TAG, "Initializing SD card");

    ESP_LOGI(TAG, "Using SPI peripheral");

    // Initialize SPI bus - using working configuration from emmc_example
    spi_bus_config_t bus_cfg = {
        .mosi_io_num = PIN_NUM_MOSI,
        .miso_io_num = PIN_NUM_MISO,
        .sclk_io_num = PIN_NUM_CLK,
        .quadwp_io_num = -1,
        .quadhd_io_num = -1,
        .max_transfer_sz = 4000,
    };

    ret = spi_bus_initialize(SPI2_HOST, &bus_cfg, SDSPI_DEFAULT_DMA);
    if (ret != ESP_OK)
    {
        ESP_LOGE(TAG, "Failed to initialize SPI bus: %s", esp_err_to_name(ret));
        return;
    }

    // Configure SPI host for SD/eMMC - use emmc_example configuration
    sdmmc_host_t host = SDSPI_HOST_DEFAULT();
    host.max_freq_khz = SDMMC_FREQ_DEFAULT; // Start with default frequency

    // Configure SPI device
    sdspi_device_config_t slot_config = SDSPI_DEVICE_CONFIG_DEFAULT();
    slot_config.gpio_cs = PIN_NUM_CS;
    slot_config.host_id = SPI2_HOST;

    ESP_LOGI(TAG, "Mounting filesystem");
    // Mount the filesystem
    ret = esp_vfs_fat_sdspi_mount(SD_MOUNT_POINT, &host, &slot_config, &mount_config, &card);

    if (ret != ESP_OK)
    {
        if (ret == ESP_FAIL)
        {
            ESP_LOGE(TAG, "Failed to mount filesystem. "
                          "If you want the SD card to be formatted, set the format_if_mount_failed option.");
        }
        else
        {
            ESP_LOGE(TAG, "Failed to initialize the card (%s). "
                          "Make sure SD card lines have pull-up resistors in place. "
                          "Check wiring: MISO=%d, MOSI=%d, SCLK=%d, CS=%d",
                     esp_err_to_name(ret), PIN_NUM_MISO, PIN_NUM_MOSI, PIN_NUM_CLK, PIN_NUM_CS);
        }
        // Clean up SPI bus on failure
        spi_bus_free(SPI2_HOST);
        return;
    }

    ESP_LOGI(TAG, "Filesystem mounted");
    ESP_LOGI(TAG, "SD card mounted successfully");
    sdmmc_card_print_info(stdout, card);
}

void record_wav(uint32_t rec_time)
{
    int flash_wr_size = 0;
    ESP_LOGI(TAG, "Opening file for recording");

    uint32_t flash_rec_time = BYTE_RATE * rec_time;
    const wav_header_t wav_header =
        WAV_HEADER_PCM_DEFAULT(flash_rec_time, 16, I2S_MIC_SAMPLE_RATE, 1);

    // Remove existing file if it exists
    struct stat st;
    if (stat(SD_MOUNT_POINT "/record.wav", &st) == 0)
    {
        ESP_LOGI(TAG, "Removing existing record.wav");
        unlink(SD_MOUNT_POINT "/record.wav");
    }

    FILE *f = fopen(SD_MOUNT_POINT "/record.wav", "w");
    if (f == NULL)
    {
        ESP_LOGE(TAG, "Failed to open file for writing");
        return;
    }

    // Write WAV header
    size_t header_written = fwrite(&wav_header, sizeof(wav_header), 1, f);
    if (header_written != 1)
    {
        ESP_LOGE(TAG, "Failed to write WAV header");
        fclose(f);
        return;
    }
    ESP_LOGI(TAG, "WAV header written successfully");

    ESP_LOGI(TAG, "Starting audio recording for %lu seconds...", (unsigned long)rec_time);

    while (flash_wr_size < flash_rec_time)
    {
        // Read I2S data
        esp_err_t read_result = i2s_channel_read(rx_handle, (char *)i2s_readraw_buff,
                                                 SAMPLE_SIZE * sizeof(int16_t), &bytes_read, 1000);

        if (read_result == ESP_OK && bytes_read > 0)
        {
            // Write data to file
            size_t written = fwrite(i2s_readraw_buff, bytes_read, 1, f);
            if (written != 1)
            {
                ESP_LOGE(TAG, "Failed to write audio data to file");
                break;
            }
            flash_wr_size += bytes_read;

            // Progress indicator every second
            if (flash_wr_size % BYTE_RATE < bytes_read)
            {
                int seconds_recorded = flash_wr_size / BYTE_RATE;
                ESP_LOGI(TAG, "Recorded %d/%lu seconds", seconds_recorded, (unsigned long)rec_time);
            }
        }
        else
        {
            ESP_LOGE(TAG, "I2S read failed: %s, bytes_read: %zu", esp_err_to_name(read_result), bytes_read);
        }
    }

    ESP_LOGI(TAG, "Recording completed!");
    fclose(f);
    ESP_LOGI(TAG, "File written to SD card: " SD_MOUNT_POINT "/record.wav");
}

void init_microphone(void)
{
#if SOC_I2S_SUPPORTS_PDM2PCM
    ESP_LOGI(TAG, "Initializing PDM microphone (PCM format)");
#else
    ESP_LOGI(TAG, "Initializing PDM microphone (raw PDM format)");
#endif

    // Create I2S channel
    i2s_chan_config_t chan_cfg = I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM_AUTO, I2S_ROLE_MASTER);
    ESP_ERROR_CHECK(i2s_new_channel(&chan_cfg, NULL, &rx_handle));

    // Configure PDM RX
    i2s_pdm_rx_config_t pdm_rx_cfg = {
        .clk_cfg = I2S_PDM_RX_CLK_DEFAULT_CONFIG(I2S_MIC_SAMPLE_RATE),
#if SOC_I2S_SUPPORTS_PDM2PCM
        .slot_cfg = I2S_PDM_RX_SLOT_PCM_FMT_DEFAULT_CONFIG(I2S_DATA_BIT_WIDTH_16BIT, I2S_SLOT_MODE_MONO),
#else
        .slot_cfg = I2S_PDM_RX_SLOT_DEFAULT_CONFIG(I2S_DATA_BIT_WIDTH_16BIT, I2S_SLOT_MODE_MONO),
#endif
        .gpio_cfg = {
            .clk = I2S_MIC_CLK_GPIO,
            .din = I2S_MIC_DATA_GPIO,
            .invert_flags = {
                .clk_inv = false,
            },
        },
    };

    ESP_ERROR_CHECK(i2s_channel_init_pdm_rx_mode(rx_handle, &pdm_rx_cfg));
    ESP_ERROR_CHECK(i2s_channel_enable(rx_handle));
    ESP_LOGI(TAG, "Microphone initialized successfully");
}

void app_main(void)
{
    ESP_LOGI(TAG, "PDM microphone recording example start");
    ESP_LOGI(TAG, "--------------------------------------");

    // Initialize SD card first
    mount_sdcard();

    // Check if SD card mount was successful
    if (card == NULL)
    {
        ESP_LOGE(TAG, "SD card initialization failed, cannot proceed with recording");
        return;
    }

    // Initialize microphone
    init_microphone();

    // Start recording
    ESP_LOGI(TAG, "Starting recording for %d seconds!", RECORDING_TIME_SEC);
    record_wav(RECORDING_TIME_SEC);

    // Clean up
    ESP_ERROR_CHECK(i2s_channel_disable(rx_handle));
    ESP_ERROR_CHECK(i2s_del_channel(rx_handle));

    // Unmount SD card
    esp_vfs_fat_sdcard_unmount(SD_MOUNT_POINT, card);
    spi_bus_free(SPI2_HOST);
    ESP_LOGI(TAG, "SD card unmounted, recording complete");
}