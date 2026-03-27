cmake_minimum_required(VERSION 3.14)
project(runner LANGUAGES CXX)

# -------------------------------
# WebView2 SDK configuration
# -------------------------------
set(WEBVIEW2_SDK_PATH "C:/webview2-sdk")

set(WEBVIEW2_INCLUDE_DIR "${WEBVIEW2_SDK_PATH}/include")
set(WEBVIEW2_LIB_DIR "${WEBVIEW2_SDK_PATH}/lib/x64")
set(WEBVIEW2_LIB "${WEBVIEW2_LIB_DIR}/WebView2LoaderStatic.lib")

if(NOT EXISTS "${WEBVIEW2_INCLUDE_DIR}/WebView2.h")
  message(FATAL_ERROR "WebView2.h not found. Check SDK path.")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(${BINARY_NAME} WIN32
  "flutter_window.cpp"
  "main.cpp"
  "utils.cpp"
  "win32_window.cpp"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
  "Runner.rc"
  "runner.exe.manifest"
  "bridge/webview2/webview_manager.cpp"
  "bridge/webview2/method_handler.cpp"
  "bridge/webview2/event_emitter.cpp"
  "bridge/webview2/request_filter.cpp"
  "bridge/ffi/rust_bridge.cpp"
)

set(RUST_PROJECT_DIR "${CMAKE_SOURCE_DIR}/../rust")
set(RUST_TARGET_DIR "${PROJECT_BINARY_DIR}/rust")
set(RUST_LIBRARY_PATH "${RUST_TARGET_DIR}/debug/netra_rust.dll")

add_custom_target(rust_bridge_build ALL
  COMMAND cargo build
    --manifest-path "${RUST_PROJECT_DIR}/Cargo.toml"
    --target-dir "${RUST_TARGET_DIR}"
  WORKING_DIRECTORY "${RUST_PROJECT_DIR}"
  COMMENT "Building Rust bridge library"
)

apply_standard_settings(${BINARY_NAME})

target_compile_definitions(${BINARY_NAME} PRIVATE
  "FLUTTER_VERSION=\"${FLUTTER_VERSION}\""
  "FLUTTER_VERSION_MAJOR=${FLUTTER_VERSION_MAJOR}"
  "FLUTTER_VERSION_MINOR=${FLUTTER_VERSION_MINOR}"
  "FLUTTER_VERSION_PATCH=${FLUTTER_VERSION_PATCH}"
  "FLUTTER_VERSION_BUILD=${FLUTTER_VERSION_BUILD}"
  "NOMINMAX"
)

target_include_directories(${BINARY_NAME} PRIVATE
  "${WEBVIEW2_INCLUDE_DIR}"
  "${CMAKE_CURRENT_SOURCE_DIR}"
  "${CMAKE_CURRENT_SOURCE_DIR}/bridge"
  "${FLUTTER_MANAGED_DIR}"
  "${FLUTTER_MANAGED_DIR}/../.."
  "${FLUTTER_MANAGED_DIR}/flutter"
)

target_link_libraries(${BINARY_NAME} PRIVATE
  flutter
  flutter_wrapper_app
  dwmapi.lib
  "${WEBVIEW2_LIB}"
)

add_dependencies(${BINARY_NAME} flutter_assemble)
add_dependencies(${BINARY_NAME} rust_bridge_build)

add_custom_command(TARGET ${BINARY_NAME} POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
    "${RUST_LIBRARY_PATH}"
    "$<TARGET_FILE_DIR:${BINARY_NAME}>/netra_rust.dll"
)
