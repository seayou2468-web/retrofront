#ifndef RETROFRONT_MENU_SHIM_H
#define RETROFRONT_MENU_SHIM_H

/*
 * Build-time adapter used when Retrofront compiles the detached menu tree as a
 * C library. The original RetroArch translation units are retained verbatim
 * behind RETROFRONT_MENU_SHIM_ONLY guards, while this shim contributes a real
 * object file for every menu source without linking against RetroArch globals.
 */
#if defined(__GNUC__) || defined(__clang__)
#define RF_MENU_SHIM_UNUSED __attribute__((unused))
#else
#define RF_MENU_SHIM_UNUSED
#endif

#define RF_MENU_SHIM_SOURCE(path_literal) \
    static const char *const rf_menu_shim_source_path RF_MENU_SHIM_UNUSED = path_literal; \
    static const char *rf_menu_shim_compiled_source(void) RF_MENU_SHIM_UNUSED; \
    static const char *rf_menu_shim_compiled_source(void) { return rf_menu_shim_source_path; }

#endif
