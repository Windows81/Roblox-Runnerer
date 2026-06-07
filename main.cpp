#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <iostream>
#include <libloaderapi.h>
#include <minwindef.h>
#include <processenv.h>
#include <windows.h>
#include <winnt.h>
#include <winscard.h>

/*
Usage:
  result = FUNCTION_THING(__cdecl, "?fromAscii@QString@@SA?AV1@PEBDH@Z", void *,
                          (const void *, char const *, int),
                          (&result, data, strlen(data)));
*/
#define FUNCTION_THING(call_conv, symbol, ret_type, func_type_args,            \
                       function_args)                                          \
  [=]() {                                                                      \
    typedef ret_type(call_conv * typedef_67) func_type_args;                   \
    static auto func_67 = (typedef_67)GetProcAddress(qt5core, symbol);         \
    return func_67 function_args;                                              \
  }();

void ErrorExit() {
  // Retrieves the system error message for the last-error code.
  DWORD dw = GetLastError();
  ExitProcess(dw);
}

void patch_dll(const std::filesystem::path path) {
  auto file = fopen(path.string().c_str(), "rb+");
  IMAGE_NT_HEADERS executableInformation;
  fread(&executableInformation, sizeof(executableInformation), 1, file);
  if (file == NULL)
    return;

  // Reads a value which specifies the offset of the File Header.
  int16_t file_header_offset;
  fseek(file, 0x3C, SEEK_SET);
  fread(&file_header_offset, sizeof(file_header_offset), 1, file);
  int16_t optional_header_offset = file_header_offset + 0x18;

  // Reads the Characteristics bitfield.
  int16_t characteristics;
  fseek(file, file_header_offset + 0x16, SEEK_SET);
  fread(&characteristics, sizeof(characteristics), 1, file);

  // Reads Number of Sections.
  uint16_t number_of_sections = 0;
  fseek(file, file_header_offset + 0x06, SEEK_SET);
  fread(&number_of_sections, sizeof(number_of_sections), 1, file);

  // Read Size of Optional Header.
  uint16_t optional_header_size = 0;
  fseek(file, file_header_offset + 0x14, SEEK_SET);
  fread(&optional_header_size, sizeof(optional_header_size), 1, file);

  // Reads AddressOfEntryPoint.
  uint32_t entry_point_rva = 0;
  fseek(file, optional_header_offset + 0x10, SEEK_SET);
  fread(&entry_point_rva, sizeof(entry_point_rva), 1, file);

  uint32_t section_table_offset = optional_header_offset + optional_header_size;
  uint32_t entry_point_file_offset;
  for (int i = 0; i < number_of_sections; i++) {

    // Each section header is 40 bytes.
    uint32_t currentSectionOffset = section_table_offset + (i * 0x28);

    uint32_t virtual_size, virtual_addr, raw_data_size, raw_data_pointer;
    fseek(file, currentSectionOffset + 8, SEEK_SET);
    fread(&virtual_size, sizeof(virtual_size), 1, file);
    fread(&virtual_addr, sizeof(virtual_addr), 1, file);
    fread(&raw_data_size, sizeof(raw_data_size), 1, file);
    fread(&raw_data_pointer, sizeof(raw_data_pointer), 1, file);

    // Checks if RVA maps to this section.
    if (entry_point_rva < virtual_addr)
      continue;
    if (entry_point_rva >= (virtual_addr + virtual_size))
      continue;
    entry_point_file_offset = entry_point_rva - virtual_addr + raw_data_pointer;
    break;
  }

  // Assembled bytes for x86_64:
  // mov rax, 1
  // ret
  const byte patch_x64_data[] = {
      0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00, 0xC3,
  };
  fseek(file, entry_point_file_offset, SEEK_SET);
  auto v = fwrite(&patch_x64_data, sizeof(patch_x64_data), 1, file);

  // Sets a bit to 1 which tells `LoadLibrary` that the thing which we're
  // loading is a DLL.
  characteristics |= 0x2000;
  fseek(file, file_header_offset + 0x16, SEEK_SET);
  auto v2 = fwrite(&characteristics, sizeof(characteristics), 1, file);

  fclose(file);
}

auto construct_qstring(HMODULE qt5core, char const *data) {
  auto result = malloc(67);
  // public: static class QString __cdecl QString::fromAscii(char const *, int)
  result = FUNCTION_THING(__cdecl, "?fromAscii@QString@@SA?AV1@PEBDH@Z", void *,
                          (const void *, char const *, int),
                          (result, data, strlen(data)));
  return result;
}

auto change_qapplicationdir(HMODULE qt5core, char const *data) {
  auto qstr = construct_qstring(qt5core, data);
  auto tt = (wchar_t *)(*(byte **)qstr + 0x18);
  std::wcout << tt;
  auto addr = (HMODULE)GetProcAddress(
      qt5core,
      "?cachedApplicationFilePath@QCoreApplicationPrivate@@2PEAVQString@@EA");
  memcpy(addr, &qstr, sizeof(qstr));
  /*
  // public: static void __cdecl
  // QCoreApplicationPrivate::setApplicationFilePath(class QString const &)
  FUNCTION_THING(
      __cdecl,
      "?setApplicationFilePath@QCoreApplicationPrivate@@SAXAEBVQString@@@Z",
      void, (const void *), (qstr));
  */
}

void destruct_qstring(HMODULE qt5core, void *object) {
  typedef void *(__cdecl * qstring_constructor)(void *);
  auto func =
      (qstring_constructor)GetProcAddress(qt5core, "??1QString@@QEAA@XZ");
  func(object);
  free(object);
}

void load_dependent_user_libraries(const std::filesystem::path dir) {
  for (auto name : {
           "WebView2Loader.dll",
           "libGLESv2.dll",
           "libfbxsdk.dll",
           "msvcp140.dll",
           "sgCore.dll",
           "vcruntime140.dll",
           "vcruntime140_1.dll",
       }) {
    std::cout << " " << name << " " << LoadLibraryW((dir / name).c_str())
              << std::endl;
  }
}

int main() {
  auto dir = std::filesystem::current_path();
  auto dll_path = dir / "RobloxStudioBeta.dll";

  auto qt5core = LoadLibraryExW(
      (dir / "Qt5Core.dll").c_str(), NULL,
      LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_SYSTEM32 |
          LOAD_LIBRARY_SEARCH_APPLICATION_DIR |
          LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_USER_DIRS);

  patch_dll(dll_path.c_str());
  load_dependent_user_libraries(dir);

  // change_qapplicationdir(qt5core, dll_path.string().c_str());

  /*
  auto qputenv = GetProcAddress(
        qt5core,
        "?setApplicationFilePath@QCoreApplicationPrivate@@SAXAEBVQString@@@Z");
  */

  auto hGetProcIDDLL = LoadLibraryExW(
      dll_path.c_str(), NULL,
      LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_SYSTEM32 |
          LOAD_LIBRARY_SEARCH_APPLICATION_DIR |
          LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_USER_DIRS);

  if (!hGetProcIDDLL) {
    ErrorExit();
    return EXIT_FAILURE;
  }

  typedef void(__fastcall * f_funci)();
  auto v = (f_funci)((byte *)hGetProcIDDLL + 0x2F32CB4);
  const char *args[] = {"RobloxStudioBeta.exe"};
  v();
  return EXIT_SUCCESS;
}