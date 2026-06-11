import Darwin

@_silgen_name("retrofront_slint_ios_main")
private func retrofrontSlintIOSMain() -> Int32

let exitCode = retrofrontSlintIOSMain()
if exitCode != 0 {
  fputs("Retrofront Slint UI exited with status \(exitCode)\n", stderr)
  Darwin.exit(exitCode)
}
