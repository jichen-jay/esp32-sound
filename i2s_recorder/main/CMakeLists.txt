idf_component_register(SRCS "i2s_recorder_main.c"
                    PRIV_REQUIRES esp_driver_i2s esp_driver_gpio
                    INCLUDE_DIRS)