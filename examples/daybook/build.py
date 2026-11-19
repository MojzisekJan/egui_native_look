"""Build the matching Daybook macOS applications or SwiftUI iOS simulator app without installing anything."""
import argparse
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('--only', choices=['all', 'egui', 'swiftui'], default='all')
    parser.add_argument('--platform', choices=['macos', 'ios-simulator'], default='macos')
    args = parser.parse_args()
    if args.platform == 'ios-simulator' and args.only != 'swiftui':
        parser.error('iOS simulator currently requires --only swiftui; egui has no iOS host yet')
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_PROFILE_DEV_DEBUG='0', CARGO_INCREMENTAL='0')
    env.setdefault('DEVELOPER_DIR', '/Applications/Xcode.app/Contents/Developer')
    target = (args.target_dir or REPO/'target').resolve()
    env['CARGO_TARGET_DIR'] = str(target)

    def bundle(kind):
        if args.platform == 'ios-simulator':
            app = out/'Daybook-SwiftUI-iOS.app'
            app.mkdir(parents=True, exist_ok=True)
            info = {'CFBundleIdentifier': 'dev.egui.daybook.swiftui.ios',
                    'CFBundleExecutable': 'Daybook', 'CFBundleName': 'Daybook',
                    'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1',
                    'CFBundleShortVersionString': '1.0', 'MinimumOSVersion': '26.0',
                    'LSRequiresIPhoneOS': True, 'UIDeviceFamily': [1, 2],
                    'UILaunchScreen': {}, 'UIApplicationSceneManifest':
                    {'UIApplicationSupportsMultipleScenes': False}}
            (app/'Info.plist').write_bytes(plistlib.dumps(info))
            shutil.copy2(HERE/'seed.json', app/'seed.json')
            return app, app/'Daybook'
        app = out/f'Daybook-{kind}.app'
        executable = app/'Contents/MacOS/Daybook'
        executable.parent.mkdir(parents=True, exist_ok=True)
        resources = app/'Contents/Resources'
        resources.mkdir(exist_ok=True)
        info = {'CFBundleIdentifier': f'dev.egui.daybook.{kind.lower()}', 'CFBundleExecutable': 'Daybook',
                'CFBundleName': f'Daybook {kind}', 'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1',
                'CFBundleShortVersionString': '1.0', 'NSHighResolutionCapable': True,
                'LSMinimumSystemVersion': '26.0', 'NSPrincipalClass': 'NSApplication'}
        (app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
        shutil.copy2(HERE/'seed.json', resources/'seed.json')
        return app, executable

    if args.only in ['all','egui']:
        subprocess.run(['cargo','build','--locked','--example','daybook'],cwd=REPO,env=env,check=True)
        app, executable = bundle('egui')
        temporary = executable.with_suffix('.new')
        shutil.copy2(target/'debug/examples/daybook',temporary)
        os.replace(temporary, executable)
        subprocess.run(['codesign','--force','--sign','-',str(app)],check=True,capture_output=True)
        print(app,flush=True)
    if args.only in ['all','swiftui']:
        app, executable = bundle('SwiftUI')
        ios = args.platform == 'ios-simulator'
        cache = target/('ios-swift-cache' if ios else 'daybook-swift-cache')
        sdk = 'iphonesimulator' if ios else 'macosx'
        triple = f'{platform.machine()}-apple-' + ('ios26.0-simulator' if ios else 'macos26.0')
        subprocess.run(['xcrun','--sdk',sdk,'swiftc','-parse-as-library','-target',triple,
                        '-module-cache-path',str(cache),str(HERE/'Daybook.swift'),'-o',str(executable)],env=env,check=True)
        subprocess.run(['codesign','--force','--sign','-',str(app)],check=True,capture_output=True)
        print(app,flush=True)

if __name__ == '__main__':
    main()
