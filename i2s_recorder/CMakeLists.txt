# T-Embed WAV Player CMakeLists.txt
cmake_minimum_required(VERSION 3.16)

# Set the supported targets
set(SUPPORTED_TARGETS esp32s3)

include($ENV{IDF_PATH}/tools/cmake/project.cmake)

project(t_embed_wav_player)