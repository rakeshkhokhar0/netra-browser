#include "bridge/ffi/rust_bridge.h"

namespace netra::bridge::ffi {

bool ShouldBlockRequest(const std::string& url,
                        const std::string& resource_type) {
  // The Rust request pipeline will own the real filtering decision. This stub
  // preserves the native bridge contract so WebView2 interception can be wired
  // without embedding filtering rules in C++.
  (void)url;
  (void)resource_type;
  return false;
}

}  // namespace netra::bridge::ffi
