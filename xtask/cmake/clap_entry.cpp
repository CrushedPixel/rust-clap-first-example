/*
 * This file re-exports the rust_clap_entry symbol from the Rust static library
 * as a standard clap_entry which clap-wrapper uses to build the base CLAP.
 */

#include "clap_entry.h"
#include <cstdint>

struct clap_version {
  uint32_t major;
  uint32_t minor;
  uint32_t revision;
};

struct clap_plugin_entry {
  clap_version version;
  bool (*init)(const char *plugin_path);
  void (*deinit)();
  const void *(*get_factory)(const char *factory_id);
};

extern "C" {

#ifdef __GNUC__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wattributes"
#endif

// The Rust library's exported symbol...
extern const clap_plugin_entry rust_clap_entry;
// ... is re-exported under the expected CLAP entry name.
CLAP_EXPORT extern const clap_plugin_entry clap_entry;
const CLAP_EXPORT struct clap_plugin_entry clap_entry = rust_clap_entry;

#ifdef __GNUC__
#pragma GCC diagnostic pop
#endif
}
