"""Build and capture independent SwiftUI and egui fixtures; never downloads simulators."""
import argparse
import json
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import time

HERE = Path(__file__).resolve().parent
BUNDLE = 'dev.egui.apple-parity-reference'
CASES = [('light', 'regular', False), ('dark', 'regular', False),
         ('light', 'regular', True), ('dark', 'regular', True),
         ('light', 'small', False), ('light', 'large', False)]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('--device', help='UDID of an existing, booted iOS simulator; omitted = macOS only')
    args = parser.parse_args()
    out = args.out.resolve()
    build = out/'build'
    build.mkdir(parents=True, exist_ok=True)
    (out/'verification.json').unlink(missing_ok=True)
    target = (args.target_dir or build/'cargo').resolve()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_PROFILE_DEV_DEBUG='0', CARGO_INCREMENTAL='0')
    env.setdefault('DEVELOPER_DIR', '/Applications/Xcode.app/Contents/Developer')
    logs = []

    def run(*command, allow_failure=False):
        # Launch Services must inherit the interactive launcher's activation context.
        # Do not run `open` through the build environment or redirected pipes.
        if command[0] == 'open':
            result = subprocess.run([str(x) for x in command], text=True)
        else:
            result = subprocess.run([str(x) for x in command], env=env, capture_output=True, text=True)
        logs.append({'command': [str(x) for x in command], 'status': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
        (out/'commands.json').write_text(json.dumps(logs, indent=2))
        if result.returncode and not allow_failure:
            raise RuntimeError(f'{command}:\n{result.stderr}\n{result.stdout}')
        return (result.stdout or '').strip()

    run('cargo', 'build', '--locked', '--manifest-path', HERE/'Cargo.toml')
    mac_app = build/'MacReference.app'
    (mac_app/'Contents/MacOS').mkdir(parents=True, exist_ok=True)
    (mac_app/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier': 'dev.egui.apple-parity-mac',
        'CFBundleName': 'SwiftUI Parity', 'CFBundleExecutable': 'NativeReference', 'CFBundlePackageType': 'APPL',
        'CFBundleVersion': '1', 'NSPrincipalClass': 'NSApplication', 'NSHighResolutionCapable': True}))
    native = mac_app/'Contents/MacOS/NativeReference'
    run('xcrun', 'swiftc', '-parse-as-library', '-target', f'{platform.machine()}-apple-macos26.0',
        HERE/'NativeReference.swift', '-module-cache-path', build/'swift-cache', '-o', native)
    if args.device:
        if platform.machine() != 'arm64':
            raise RuntimeError('The Rust iOS probe currently requires an Apple silicon host.')
        run('cargo', 'build', '--locked', '--manifest-path', HERE/'ios-probe/Cargo.toml', '--target', 'aarch64-apple-ios-sim')
        sdk = run('xcrun', '--sdk', 'iphonesimulator', '--show-sdk-path')
        app = build/'NativeReference.app'
        app.mkdir(exist_ok=True)
        info = {'CFBundleIdentifier': BUNDLE, 'CFBundleExecutable': 'NativeReference', 'CFBundleName': 'SwiftUI Reference',
                'CFBundlePackageType': 'APPL', 'CFBundleShortVersionString': '1.0', 'CFBundleVersion': '1',
                'LSRequiresIPhoneOS': True, 'UIDeviceFamily': [1, 2], 'UILaunchScreen': {},
                'UIApplicationSceneManifest': {'UIApplicationSupportsMultipleScenes': False}}
        (app/'Info.plist').write_bytes(plistlib.dumps(info))
        run('xcrun', '--sdk', 'iphonesimulator', 'swiftc', '-parse-as-library', '-D', 'RUST_PROBE',
            '-target', 'arm64-apple-ios26.0-simulator', '-sdk', sdk, '-module-cache-path', build/'swift-ios-cache',
            HERE/'NativeReference.swift', target/'aarch64-apple-ios-sim/debug/libapple_parity_ios_probe.a',
            '-framework', 'CoreText', '-framework', 'CoreFoundation', '-o', app/'NativeReference')
        run('codesign', '--force', '--sign', '-', app)
        run('xcrun', 'simctl', 'terminate', args.device, BUNDLE, allow_failure=True)
        run('xcrun', 'simctl', 'install', args.device, app)
        documents = Path(run('xcrun', 'simctl', 'get_app_container', args.device, BUNDLE, 'data'))/'Documents'

    for scheme, size, disabled in CASES:
        options = ['--scheme', scheme, '--size', size] + (['--disabled'] if disabled else [])
        suffix = f'{scheme}-{size}' + ('-disabled' if disabled else '')
        mac_out = out/('mac-'+suffix)
        for name in ['reference.json', 'reference.png']:
            (mac_out/name).unlink(missing_ok=True)
        run('open', '-n', '-W', mac_app, '--args', '--out', mac_out, '--quit', *options)
        state = json.loads((mac_out/'reference.json').read_text())['metadata']
        assert state['actual_app_active'] and state['actual_key_window'], state
        run(target/'debug/apple-parity', '--out', mac_out, '--look', 'mac', '--scale', str(state['scale']), *options)
        if args.device:
            run('xcrun', 'simctl', 'terminate', args.device, BUNDLE, allow_failure=True)
            for name in ['reference.json', 'reference.png', 'rust-probe.json']:
                (documents/name).unlink(missing_ok=True)
            run('xcrun', 'simctl', 'launch', args.device, BUNDLE, *options)
            deadline = time.monotonic()+25
            while not all((documents/name).exists() for name in ['reference.png', 'rust-probe.json']):
                if time.monotonic() > deadline:
                    raise TimeoutError(f'No complete iOS capture for {suffix}')
                time.sleep(.2)
            ios_out = out/('ios-'+suffix)
            ios_out.mkdir(parents=True, exist_ok=True)
            for name in ['reference.json', 'reference.png', 'rust-probe.json']:
                shutil.copy2(documents/name, ios_out/name)
            probe = json.loads((ios_out/'rust-probe.json').read_text())
            assert probe['target'] == 'ios' and probe['installed'] and probe['system_fonts'], probe
            ios_state = json.loads((ios_out/'reference.json').read_text())['metadata']
            run(target/'debug/apple-parity', '--out', ios_out, '--look', 'ios', '--scale', str(ios_state['scale']), *options)
        print(f'Captured {suffix}', flush=True)

    (out/'mac-input/reference.json').unlink(missing_ok=True)
    run('open', '-n', '-W', mac_app, '--args', '--probe', '--quit', '--out', out/'mac-input')
    input_result = json.loads((out/'mac-input/reference.json').read_text())
    assert input_result['metadata']['actual_app_active'] and input_result['metadata']['actual_key_window']
    traces = input_result['trace']
    assert all(t['app_active'] and t['key_window'] for t in traces), 'Inactive native input trace'
    expected = {'return_default': 1, 'command_return': 0, 'shift_return': 1, 'button_click': 1,
                'button_leave_return_release': 1, 'button_leave_release': 0, 'checkbox_return': 1, 'text_return': 1}
    observed = [trace['case'] for trace in traces]
    assert all(observed.count(case) == 1 for case in expected), observed
    for trace in traces:
        if trace['case'] in expected:
            assert trace['after']['count']-trace['before']['count'] == expected[trace['case']], trace
    keyboard_out = out/'mac-keyboard'
    (keyboard_out/'reference.json').unlink(missing_ok=True)
    run('open', '-n', '-W', mac_app, '--args', '--full-keyboard-probe', '--probe', '--quit', '--out', keyboard_out)
    keyboard = json.loads((keyboard_out/'reference.json').read_text())
    assert keyboard['metadata']['actual_app_active'] and keyboard['metadata']['actual_key_window']
    assert keyboard['metadata']['full_keyboard_access']
    assert all(t['app_active'] and t['key_window'] for t in keyboard['trace']), 'Inactive native keyboard trace'
    traces_by_name = {trace['case']: trace for trace in keyboard['trace']}
    for case, control in [('focus_switch','switch'), ('focus_segments','segmented')]:
        assert traces_by_name[case]['confirmed_control'] == control, traces_by_name[case]
    switch = traces_by_name['focused_switch_space']
    assert switch['after']['switched'] != switch['before']['switched'], switch
    for case in ['focused_switch_return', 'focused_segments_return']:
        trace = traces_by_name[case]
        assert trace['after']['count'] == trace['before']['count']+1, trace
        for key in ['switched', 'segment']:
            assert trace['after'][key] == trace['before'][key], trace
    picker = traces_by_name['picker_return']
    assert picker['after']['choice'] == 2 and picker['after']['count'] == picker['before']['count'], picker
    escape = traces_by_name['picker_escape']
    assert escape['after']['choice'] == escape['before']['choice'] and escape['after']['count'] == escape['before']['count'], escape
    (out/'verification.json').write_text(json.dumps({'native_cases': len(CASES)*(2 if args.device else 1),
        'mac_input_assertions': len(expected), 'mac_keyboard_assertions': 7, 'ios_rust_font_probe': bool(args.device),
        'note': 'egui is rendered on the host; the iOS Rust probe exercises fonts and input geometry, not UIKit event delivery.'}, indent=2))
    print('Native captures and assertions passed.', flush=True)

if __name__ == '__main__':
    main()
