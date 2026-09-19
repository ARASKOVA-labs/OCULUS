cmake_minimum_required(VERSION 3.16)
project(oculus_native C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(PkgConfig REQUIRED)
pkg_check_modules(TESSERACT tesseract)
pkg_check_modules(LEPTONICA lept)

add_library(oculus_native STATIC
    cpp/oculus_engine.cc
)

target_include_directories(oculus_native PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

if(TESSERACT_FOUND AND LEPTONICA_FOUND)
    target_include_directories(oculus_native PUBLIC
        ${TESSERACT_INCLUDE_DIRS}
        ${LEPTONICA_INCLUDE_DIRS}
    )
    target_link_directories(oculus_native PUBLIC
        ${TESSERACT_LIBRARY_DIRS}
        ${LEPTONICA_LIBRARY_DIRS}
        /opt/homebrew/lib
    )
    target_link_libraries(oculus_native PUBLIC
        ${TESSERACT_LIBRARIES}
        ${LEPTONICA_LIBRARIES}
    )
    target_compile_definitions(oculus_native PUBLIC OCULUS_HAS_TESSERACT=1)
endif()

if(APPLE)
    find_library(CORE_GRAPHICS_FRAMEWORK CoreGraphics REQUIRED)
    target_link_libraries(oculus_native PUBLIC ${CORE_GRAPHICS_FRAMEWORK})
endif()

enable_testing()
add_executable(test_native_engine test/test_engine.cc)
target_link_libraries(test_native_engine PRIVATE oculus_native)
add_test(NAME NativeEngineTest COMMAND test_native_engine)
