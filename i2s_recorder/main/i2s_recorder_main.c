/*
 * T-Embed WAV Player - Plays embedded WAV file through onboard speaker
 * SPDX-License-Identifier: Unlicense OR CC0-1.0
 */

#include <stdio.h>
#include <string.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "driver/i2s_std.h"
#include "driver/gpio.h"
#include "esp_log.h"
#include "esp_system.h"

static const char *TAG = "t_embed_player";

// T-Embed I2S pins for onboard speaker/amplifier
#define I2S_BCLK_IO 7 // Bit clock
#define I2S_WS_IO 15  // Word select
#define I2S_DOUT_IO 6 // Data out to speaker
#define I2S_SAMPLE_RATE 16000
#define I2S_NUM I2S_NUM_0

// Speaker enable/power pin (if exists)
#define SPEAKER_EN_GPIO 46 // Enable pin for speaker amplifier

static i2s_chan_handle_t tx_handle = NULL;

// Import embedded WAV file - we'll create a simple tone instead
// You can replace this with actual WAV data
static const uint16_t sine_wave_data[] = {
    // Simple 440Hz sine wave samples at 16kHz (about 36 samples per cycle)
    32767, 35287, 37615, 39675, 41415, 42794, 43784, 44365,
    44524, 44258, 43571, 42472, 41079, 39411, 37496, 35363,
    33042, 30571, 27985, 25323, 22627, 19935, 17289, 14727,
    12288, 10010, 7927, 6074, 4481, 3180, 2195, 1548,
    1257, 1335, 1790, 2625, 3837, 5418, 7355, 9629,
    12217, 15092, 18224, 21582, 25133, 28845, 32683, 36613,
    40600, 44609, 48604, 52549, 56408, 60147, 63732, 67128,
    70303, 73225, 75864, 78192, 80183, 81814, 83062, 83908,
    84335, 84329, 83877, 82971, 81601, 79764, 77456, 74677,
    71430, 67720, 63556, 58950, 53916, 48471, 42635, 36429,
    29877, 23006, 15844, 8423, 772, -7093, -15130, -23300,
    -31564, -39886, -48228, -56554, -64826, -73007, -81062,
    -88954, -96650, -104114, -111313, -118214, -124785, -130995,
    -136815, -142215, -147168, -151648, -155631, -159093, -162015,
    -164375, -166157, -167345, -167923, -167880, -167205, -165890,
    -163929, -161316, -158049, -154125, -149544, -144307, -138419,
    -131884, -124710, -116904, -108477, -99442, -89813, -79605,
    -68837, -57528, -45700, -33375, -20576, -7327, 6349, 20427,
    34923, 49808, 65054, 80632, 96514, 112671, 129073, 145691,
    162494, 179452, 196534, 213710, 230949, 248219, 265489, 282727,
    299903, 316986, 333944, 350746, 367361, 383757, 399904, 415769,
    431322, 446531, 461365, 475793, 489783, 503304, 516326, 528818,
    540751, 552094, 562819, 572897, 582300, 590999, 598968, 606179,
    612606, 618224, 623007, 626932, 629976, 632118, 633338, 633616,
    632935, 631278, 628629, 624974, 620300, 614593, 607843, 600039,
    591174, 581240, 570230, 558138, 544959, 530690, 515329, 498875,
    481329, 462692, 442968, 422162, 400280, 377329, 353318, 328256,
    302154, 275024, 246880, 217735, 187605, 156507, 124460, 91483,
    57596, 22821, -12819, -49296, -86574, -124622, -163408, -202900,
    -243063, -283862, -325260, -367220, -409704, -452674, -496090,
    -539911, -584096, -628603, -673389, -718410, -763623, -808983,
    -854444, -899962, -945491, -990986, -1036400, -1081688, -1126803,
    -1171699, -1216329, -1260648, -1304609, -1348167, -1391277, -1433892,
    -1475968, -1517460, -1558322, -1598508, -1637974, -1676673, -1714562,
    -1751596, -1787731, -1822923, -1857128, -1890302, -1922401, -1953382,
    -1983201, -2011817, -2039188, -2065270, -2090022, -2113402, -2135369,
    -2155881, -2174897, -2192377, -2208280, -2222566, -2235195, -2246127,
    -2255323, -2262742, -2268346, -2272096, -2273954, -2273881, -2271840,
    -2267793, -2261703, -2253534, -2243248, -2230809, -2216181, -2199329,
    -2180218, -2158813, -2135079, -2108982, -2080488, -2049563, -2016174,
    -1980288, -1941873, -1900897, -1857328, -1811135, -1762287, -1710754,
    -1656505, -1599511, -1539744, -1477176, -1411779, -1343528, -1272397,
    -1198362, -1121398, -1041482, -958591, -872703, -783796, -691849,
    -596843, -498759, -397579, -293284, -185857, -75281, 37456, 152366,
    269453, 388732, 510216, 633918, 759852, 888031, 1018467, 1151173,
    1286161, 1423444, 1563033, 1704940, 1849177, 1995754, 2144683, 2295975,
    2449640, 2605689, 2764133, 2924982, 3088246, 3253935, 3422058, 3592625,
    3765646, 3941129, 4119084, 4299520, 4482445, 4667868, 4855798, 5046244,
    5239214, 5434717, 5632761, 5833354, 6036505, 6242223, 6450516, 6661393,
    6874862, 7090932, 7309610, 7530905, 7754825, 7981379, 8210575, 8442420,
    8676923, 8914092, 9153935, 9396460, 9641675, 9889587, 10140205, 10393537,
    10649591, 10908374, 11169895, 11434161, 11701181, 11970963, 12243514,
    12518842, 12796955, 13077860, 13361566, 13648079, 13937408, 14229560,
    14524543, 14822364, 15123030, 15426549, 15732928, 16042175, 16354297,
    16669302, 16987197, 17307990, 17631688, 17958299, 18287831, 18620291,
    18955687, 19294027, 19635318, 19979568, 20326785, 20676977, 21030151,
    21386315, 21745477, 22107644, 22472824, 22841025, 23212254, 23586520,
    23963830, 24344192, 24727614, 25114103, 25503667, 25896314, 26292052,
    26690888, 27092831, 27497889, 27906069, 28317379, 28731827, 29149420,
    29570166, 29994073, 30421148, 30851400, 31284837, 31721467, 32161298};

#define SINE_WAVE_SIZE (sizeof(sine_wave_data) / sizeof(sine_wave_data[0]))

void i2s_driver_init(void)
{
    ESP_LOGI(TAG, "Initializing I2S for T-Embed speaker");

    // Create I2S channel
    i2s_chan_config_t chan_cfg = I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM, I2S_ROLE_MASTER);
    chan_cfg.auto_clear = true;
    ESP_ERROR_CHECK(i2s_new_channel(&chan_cfg, &tx_handle, NULL));

    // Configure I2S for T-Embed speaker
    i2s_std_config_t std_cfg = {
        .clk_cfg = I2S_STD_CLK_DEFAULT_CONFIG(I2S_SAMPLE_RATE),
        .slot_cfg = I2S_STD_MSB_SLOT_DEFAULT_CONFIG(I2S_DATA_BIT_WIDTH_16BIT, I2S_SLOT_MODE_MONO),
        .gpio_cfg = {
            .mclk = I2S_GPIO_UNUSED,
            .bclk = I2S_BCLK_IO,
            .ws = I2S_WS_IO,
            .dout = I2S_DOUT_IO,
            .din = I2S_GPIO_UNUSED,
            .invert_flags = {
                .mclk_inv = false,
                .bclk_inv = false,
                .ws_inv = false,
            },
        },
    };

    ESP_ERROR_CHECK(i2s_channel_init_std_mode(tx_handle, &std_cfg));
    ESP_ERROR_CHECK(i2s_channel_enable(tx_handle));

    ESP_LOGI(TAG, "I2S initialized: BCLK=%d, WS=%d, DOUT=%d", I2S_BCLK_IO, I2S_WS_IO, I2S_DOUT_IO);
}

void speaker_enable(void)
{
    // Enable speaker amplifier if there's an enable pin
    ESP_LOGI(TAG, "Enabling speaker amplifier");

    gpio_config_t io_conf = {
        .pin_bit_mask = (1ULL << SPEAKER_EN_GPIO),
        .mode = GPIO_MODE_OUTPUT,
        .pull_up_en = GPIO_PULLUP_DISABLE,
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .intr_type = GPIO_INTR_DISABLE,
    };

    esp_err_t ret = gpio_config(&io_conf);
    if (ret == ESP_OK)
    {
        gpio_set_level(SPEAKER_EN_GPIO, 1); // Enable speaker
        ESP_LOGI(TAG, "Speaker amplifier enabled on GPIO %d", SPEAKER_EN_GPIO);
    }
    else
    {
        ESP_LOGW(TAG, "Failed to configure speaker enable pin: %s", esp_err_to_name(ret));
    }
}

void play_tone_task(void *args)
{
    ESP_LOGI(TAG, "Starting audio playback task");

    size_t bytes_written = 0;
    int play_count = 0;

    // Play the tone multiple times
    while (play_count < 10)
    { // Play 10 times
        ESP_LOGI(TAG, "Playing tone sequence %d/10", play_count + 1);

        // Play the sine wave multiple times to make it longer
        for (int repeat = 0; repeat < 50; repeat++)
        {
            esp_err_t ret = i2s_channel_write(tx_handle, sine_wave_data,
                                              SINE_WAVE_SIZE * sizeof(uint16_t),
                                              &bytes_written, portMAX_DELAY);

            if (ret != ESP_OK)
            {
                ESP_LOGE(TAG, "I2S write failed: %s", esp_err_to_name(ret));
                break;
            }

            // Small delay between repeats
            vTaskDelay(pdMS_TO_TICKS(10));
        }

        play_count++;

        // Pause between each play sequence
        ESP_LOGI(TAG, "Pausing between sequences...");
        vTaskDelay(pdMS_TO_TICKS(1000));
    }

    ESP_LOGI(TAG, "Audio playback completed");

    // Clean up
    ESP_ERROR_CHECK(i2s_channel_disable(tx_handle));
    ESP_ERROR_CHECK(i2s_del_channel(tx_handle));

    vTaskDelete(NULL);
}

void test_different_pins(void)
{
    ESP_LOGI(TAG, "Testing different I2S pin configurations for T-Embed speaker");

    // Different possible I2S pin configurations for T-Embed
    struct
    {
        int bclk, ws, dout;
        const char *name;
    } pin_configs[] = {
        {7, 15, 6, "Config A (7/15/6)"},
        {4, 5, 6, "Config B (4/5/6)"},
        {12, 13, 14, "Config C (12/13/14)"},
        {2, 3, 4, "Config D (2/3/4)"},
        {18, 19, 20, "Config E (18/19/20)"},
    };

    int num_configs = sizeof(pin_configs) / sizeof(pin_configs[0]);

    for (int i = 0; i < num_configs; i++)
    {
        ESP_LOGI(TAG, "Trying %s", pin_configs[i].name);

        // Clean up previous I2S if exists
        if (tx_handle)
        {
            i2s_channel_disable(tx_handle);
            i2s_del_channel(tx_handle);
            tx_handle = NULL;
        }
        vTaskDelay(pdMS_TO_TICKS(100));

        // Try current pin configuration
        i2s_chan_config_t chan_cfg = I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM, I2S_ROLE_MASTER);
        chan_cfg.auto_clear = true;

        esp_err_t ret = i2s_new_channel(&chan_cfg, &tx_handle, NULL);
        if (ret != ESP_OK)
        {
            ESP_LOGW(TAG, "Failed to create I2S channel: %s", esp_err_to_name(ret));
            continue;
        }

        i2s_std_config_t std_cfg = {
            .clk_cfg = I2S_STD_CLK_DEFAULT_CONFIG(I2S_SAMPLE_RATE),
            .slot_cfg = I2S_STD_MSB_SLOT_DEFAULT_CONFIG(I2S_DATA_BIT_WIDTH_16BIT, I2S_SLOT_MODE_MONO),
            .gpio_cfg = {
                .mclk = I2S_GPIO_UNUSED,
                .bclk = pin_configs[i].bclk,
                .ws = pin_configs[i].ws,
                .dout = pin_configs[i].dout,
                .din = I2S_GPIO_UNUSED,
                .invert_flags = {
                    .mclk_inv = false,
                    .bclk_inv = false,
                    .ws_inv = false,
                },
            },
        };

        ret = i2s_channel_init_std_mode(tx_handle, &std_cfg);
        if (ret == ESP_OK)
        {
            ret = i2s_channel_enable(tx_handle);
            if (ret == ESP_OK)
            {
                ESP_LOGI(TAG, "✓ %s - I2S initialized successfully!", pin_configs[i].name);

                // Play a short test tone
                size_t bytes_written;
                for (int j = 0; j < 10; j++)
                {
                    i2s_channel_write(tx_handle, sine_wave_data,
                                      SINE_WAVE_SIZE * sizeof(uint16_t),
                                      &bytes_written, 100);
                    vTaskDelay(pdMS_TO_TICKS(50));
                }

                ESP_LOGI(TAG, "Test tone played on %s", pin_configs[i].name);
                vTaskDelay(pdMS_TO_TICKS(1000));
            }
            else
            {
                ESP_LOGW(TAG, "✗ %s - Failed to enable I2S: %s", pin_configs[i].name, esp_err_to_name(ret));
            }
        }
        else
        {
            ESP_LOGW(TAG, "✗ %s - Failed to init I2S: %s", pin_configs[i].name, esp_err_to_name(ret));
        }
    }
}

void app_main(void)
{
    ESP_LOGI(TAG, "T-Embed WAV Player Example");
    ESP_LOGI(TAG, "=========================");
    ESP_LOGI(TAG, "Playing embedded audio through onboard speaker");

    // Enable speaker amplifier
    speaker_enable();

    // Test different pin configurations to find the working one
    test_different_pins();

    ESP_LOGI(TAG, "Pin testing complete. Check logs for working configuration.");
    ESP_LOGI(TAG, "You can now modify the main code to use the working pins and play longer audio.");
}