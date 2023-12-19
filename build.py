import subprocess
import os, shutil


def main():
    result = subprocess.run(["cargo", "build", "--release"])

    if result.returncode == 0:
        build_dir = os.path.join("build")
        res_dir = os.path.join("res")
        executable = os.path.join("target", "release", "raytracer_wgpu.exe")
        shutil.copytree(res_dir, os.path.join(build_dir, res_dir), dirs_exist_ok=True)
        shutil.copy(executable, build_dir)
    else:
        print("Build failed")


if __name__ == "__main__":
    main()
