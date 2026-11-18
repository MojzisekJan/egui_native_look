"""Real IBus and Orca tests on an isolated display and session bus."""
import argparse, ast, contextlib, json, os, pathlib, select, signal, subprocess, time, xml.etree.ElementTree as ET
import importlib.util
HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('parity', HERE / 'run.py')
parity = importlib.util.module_from_spec(spec)
spec.loader.exec_module(parity)

def wait(read, predicate=lambda x: bool(x), seconds=15):
    return parity.wait_for(read, predicate, seconds)

class Live(parity.Driver):

    def __init__(self, out, runtime, binary, ime_runtime):
        super().__init__(out, runtime, binary)
        self.ime_runtime = ime_runtime
        self.services = []
        self.logs = []
        self.results = []
        self.env.update(GTK_A11Y='atspi', NO_AT_BRIDGE='0', GTK_IM_MODULE='ibus', XMODIFIERS='@im=ibus', IBUS_USE_PORTAL='0', IME_DIALOG='1', XDG_DATA_HOME=str(out / 'data'), IBUS_COMPONENT_PATH=str(out / 'components'), M17NDIR=str(ime_runtime / 'usr/share/m17n'), GSETTINGS_SCHEMA_DIR=str(ime_runtime / 'usr/share/glib-2.0/schemas'))
        self.env['LD_LIBRARY_PATH'] = str(ime_runtime / 'usr/lib/x86_64-linux-gnu') + ':' + self.env['LD_LIBRARY_PATH']
        self.env.pop('AT_SPI_BUS_ADDRESS', None)
        self.env.pop('IBUS_USE_PORTAL', None)
        self.env['ACCESSKIT_UNIX_ENABLED'] = '1'
        self.engine_history = []
        self.started = False

    def run(self, args):
        return subprocess.check_output(list(map(str, args)), env=self.env, text=True, stderr=subprocess.STDOUT, timeout=8).strip()

    def start(self, name, args, stdout=None):
        log = (self.out / (name + '.log')).open('w')
        self.logs.append(log)
        proc = subprocess.Popen(list(map(str, args)), env=self.env, stdout=stdout or log, stderr=log, start_new_session=True)
        self.services.append(proc)
        return proc

    @contextlib.contextmanager
    def infrastructure(self):
        if self.out.exists() and any(self.out.iterdir()):
            raise ValueError('Use an empty output directory for an isolated live run')
        self.out.mkdir(mode=0o700, parents=True, exist_ok=True)
        self.out.chmod(0o700)
        self.started = True
        (self.out / 'components').mkdir(exist_ok=True)
        (self.out / 'orca-prefs').mkdir(exist_ok=True)
        engine = self.ime_runtime / 'usr/libexec/ibus-engine-m17n'
        xml = ET.fromstring(subprocess.check_output([str(engine), '--xml'], env=self.env, text=True, stderr=subprocess.DEVNULL, timeout=8))
        component = ET.Element('component')
        for k, v in [('name', 'org.freedesktop.IBus.M17N'), ('description', 'Local m17n test'), ('exec', str(engine) + ' --ibus'), ('version', '1.4.28'), ('author', 'm17n'), ('license', 'GPL'), ('homepage', 'https://github.com/ibus/ibus-m17n'), ('textdomain', 'ibus-m17n')]:
            ET.SubElement(component, k).text = v
        engines = ET.SubElement(component, 'engines')
        for node in xml:
            if node.findtext('name') in ['m17n:t:latn-post', 'm17n:t:unicode']:
                engines.append(node)
        assert len(engines) == 2
        ET.ElementTree(component).write(self.out / 'components/m17n.xml', encoding='utf-8')
        config = self.out / 'session.conf'
        config.write_text('<busconfig><type>session</type><listen>unix:tmpdir=' + str(self.out) + '</listen><auth>EXTERNAL</auth><policy context="default"><allow send_destination="*" eavesdrop="true"/><allow eavesdrop="true"/><allow own="*"/></policy></busconfig>')
        try:
            bus = self.start('session-bus', ['dbus-daemon', '--nofork', '--config-file=' + str(config), '--print-address=1'], subprocess.PIPE)
            if not select.select([bus.stdout], [], [], 8)[0]:
                raise RuntimeError('Private bus did not start')
            self.env['DBUS_SESSION_BUS_ADDRESS'] = bus.stdout.readline().decode().strip()
            with self.display():
                self.start('atspi', ['/usr/libexec/at-spi-bus-launcher', '--launch-immediately'])
                answer = wait(lambda: self.run(['gdbus', 'call', '--session', '--dest', 'org.a11y.Bus', '--object-path', '/org/a11y/bus', '--method', 'org.a11y.Bus.GetAddress']))
                self.env['AT_SPI_BUS_ADDRESS'] = ast.literal_eval(answer)[0]
                self.start('ibus', ['ibus-daemon', '--verbose', '--cache=refresh', '--xim', '--panel=disable', '--config=disable', '--emoji-extension=disable'])

                def address():
                    for p in (self.out / 'config/ibus/bus').glob('*'):
                        for line in p.read_text().splitlines():
                            if line.startswith('IBUS_ADDRESS='):
                                return line.split('=', 1)[1]
                self.env['IBUS_ADDRESS'] = wait(address)
                (self.out / 'engine-list.txt').write_text(self.run(['ibus', 'list-engine']))
                self.start('orca', ['orca', '--debug-file', self.out / 'orca-debug.log', '--user-prefs', self.out / 'orca-prefs', '--disable', 'speech', '--disable', 'braille'])
                os.environ.update(self.env)
                time.sleep(2)
                yield
        finally:
            for proc in reversed(self.services):
                if proc.poll() is not None:
                    continue
                try:
                    os.killpg(proc.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    proc.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait()
            for log in self.logs:
                log.close()

    def engine(self, name):
        os.environ.update(self.env)
        import gi
        gi.require_version('IBus', '1.0')
        from gi.repository import IBus
        IBus.init()
        bus = IBus.Bus.new()

        def switch():
            if not bus.is_connected() or not bus.set_global_engine(name):
                return None
            current = bus.get_global_engine()
            return current.get_name() if current else None
        actual = wait(switch, lambda x: x == name)
        time.sleep(0.3)
        self.engine_history.append(actual)
        (self.out / 'engines-used.json').write_text(json.dumps(self.engine_history, indent=2) + '\n')

    def observe(self, app, case):
        state = self.command(app)
        result = dict(app=app, case=case, state=state)
        if case.startswith('dialog'):
            from gi.repository import IBus
            bus = IBus.Bus.new()
            current = bus.get_global_engine()
            result['ibus'] = {'engine': current.get_name() if current else None, 'enabled': bus.is_global_engine_enabled(), 'current_context': bus.current_input_context()}

        self.results.append(result)
        (self.out / 'live-results.json').write_text(json.dumps(self.results, indent=2, ensure_ascii=False) + '\n')
        print(app, case, repr(state.get('entry')), state.get('number'), state.get('dialog_open', state.get('dialog')), state.get('dialog_save'), flush=True)
        self.disk()
        return state

    def disk(self):
        import shutil
        free = shutil.disk_usage(self.out).free
        if free < 6 * 1024 ** 3:
            raise RuntimeError('Stopping tests: less than 6 GiB free')

    def tree(self, app, case):
        import pyatspi
        nodes = []

        def walk(node, depth=0):
            if depth > 15 or len(nodes) > 2000:
                return
            try:
                states = node.getState()
                row = dict(name=node.name, role=node.getRoleName(), depth=depth, focused=states.contains(pyatspi.STATE_FOCUSED), enabled=states.contains(pyatspi.STATE_ENABLED), sensitive=states.contains(pyatspi.STATE_SENSITIVE))
                try:
                    row['text'] = node.queryText().getText(0, -1)
                except (NotImplementedError, AttributeError):
                    pass
                try:
                    value = node.queryValue()
                    row['value'] = [value.currentValue, value.minimumValue, value.maximumValue, value.minimumIncrement]
                except (NotImplementedError, AttributeError):
                    pass
                nodes.append(row)
                for child in node:
                    walk(child, depth + 1)
            except Exception as error:
                nodes.append(dict(error=str(error), depth=depth))
        for application in pyatspi.Registry.getDesktop(0):
            walk(application)
        (self.out / (app + '-' + case + '-tree.json')).write_text(json.dumps(nodes, indent=2, ensure_ascii=False) + '\n')
        print(app, case, 'AT-SPI nodes', len(nodes), flush=True)
        return nodes

    def exercise(self, app):
        self.env.pop('SPIN_ROW', None)
        with self.app(app):
            self.engine('m17n:t:latn-post')
            self.command(app, text='', focus='entry')
            self.key('ctrl+a')
            self.key('BackSpace')
            self.key('c')
            self.key('asciitilde')
            self.observe(app, 'entry-preedit')
            self.key('Return')
            self.observe(app, 'entry-commit')
            self.key('a')
            self.key('apostrophe')
            self.observe(app, 'entry-second-preedit')
            self.key('Escape')
            self.observe(app, 'entry-escape')
            self.tree(app, 'entry')
            self.engine('m17n:t:unicode')
            self.command(app, number=4, focus='number')
            self.key('ctrl+a')
            self.key('BackSpace')
            self.key('ctrl+u')
            self.key('0')
            self.key('0')
            self.key('3')
            self.key('7')
            self.observe(app, 'number-preedit')
            self.key('space')
            self.observe(app, 'number-ime-commit')
            self.key('Return')
            self.observe(app, 'number-commit')
            self.tree(app, 'number')
        # Each IME scenario gets a fresh app to avoid cross-engine preedit state.
        with self.app(app):
            self.engine('m17n:t:latn-post')
            self.command(app, focus='dialog')
            self.key('Return')
            time.sleep(0.6)
            self.key('c')
            self.key('asciitilde')
            self.tree(app, 'dialog')
            self.observe(app, 'dialog-preedit')
            self.key('Return')
            self.observe(app, 'dialog-ime-commit')
            self.key('Return')
            time.sleep(1.0)
            self.observe(app, 'dialog-submit')
            self.command(app, focus='dialog')
            self.key('Return')
            time.sleep(0.5)
            self.observe(app, 'dialog-reopen')
            self.key('a')
            self.key('apostrophe')
            self.key('Escape')
            self.observe(app, 'dialog-first-escape')
            self.key('Escape')
            self.observe(app, 'dialog-second-escape')
            self.snapshot(app + '-after-dialog')
            self.command(app, focus='button')
            for _ in range(9):
                self.key('Tab')
            self.tree(app, 'navigation')

    def find(self, name, role=None):
        import pyatspi

        def walk(node, depth=0):
            if depth > 15:
                return None
            try:
                if node.name == name and (role is None or node.getRoleName() == role):
                    return node
                for child in node:
                    result = walk(child, depth + 1)
                    if result is not None:
                        return result
            except Exception:
                pass
            return None
        return wait(lambda: next((found for app in pyatspi.Registry.getDesktop(0) if (found := walk(app)) is not None), None))

    def actions(self, app):
        with self.app(app):
            self.command(app, focus='number')
            self.find('Number', 'spin button').queryValue().currentValue = 6.0
            self.observe(app, 'atspi-number-set')
            self.find('Number', 'spin button').queryValue().currentValue = 9999.0
            self.observe(app, 'atspi-number-clamp')
            self.command(app, focus='slider')
            self.find('Slider', 'slider').queryValue().currentValue = 75.0
            self.observe(app, 'atspi-slider-set')
            if app == 'egui':
                import math
                for control, role, field, bounds in [('number', 'spin button', 'number', (-10, 10)), ('slider', 'slider', 'slider', (-10, 100))]:
                    for label, value in [('nan', float('nan')), ('positive-infinity', float('inf')), ('negative-infinity', -float('inf'))]:
                        try:self.find('Number' if control == 'number' else 'Slider', role).queryValue().currentValue = value
                        except Exception as error:(self.out / (control + '-' + label + '-action.txt')).write_text(str(error) + '\n')
                        state = self.observe(app, 'atspi-' + control + '-' + label)
                        assert math.isfinite(state[field]) and bounds[0] <= state[field] <= bounds[1]

            self.command(app, focus='entry')
            self.find('Text field', 'text' if app == 'gtk' else 'entry').queryEditableText().setTextContents('č7')
            self.observe(app, 'atspi-editable-text')
            self.command(app, text='S3cret-č', password=True, focus='entry')
            tree = self.tree(app, 'password')
            assert any(node.get('name') == 'Text field' and node.get('role') == ('text' if app == 'gtk' else 'password text') and 'text' in node for node in tree), 'password editor missing from accessible tree'
            assert not any(('S3cret' in str(node) for node in tree)), 'password escaped into the accessible tree'
            self.command(app, password=False, text='Example', number=4, controls_enabled=False)
            try:
                self.find('Number', 'spin button').queryValue().currentValue = 8.0
            except Exception as error:
                (self.out / (app + '-disabled-action.txt')).write_text(str(error) + '\n')
            self.observe(app, 'atspi-disabled-number')
            self.command(app, controls_enabled=True, reset=True, focus='check')
            action = self.find('Notifications', 'check box').queryAction()
            count = action.nActions
            (self.out / (app + '-checkbox-actions.json')).write_text(json.dumps([action.getName(i) for i in range(count)]) + '\n')
            if count:
                action.doAction(0)
            self.observe(app, 'atspi-checkbox-action')
            self.engine('m17n:t:latn-post')
            for target in ['switch', 'button']:
                self.command(app, text='prefix|', focus='entry')
                self.key('End')
                self.key('c')
                self.key('asciitilde')
                self.observe(app, target + '-preedit')
                box = self.state(app)['widgets'][target]
                width = box.get('bounds_width', box['width'])
                self.click((round(box['x'] + width / 2), round(box['y'] + box['height'] / 2)))
                self.observe(app, target + '-focus-transfer')
                if target == 'switch':
                    self.key('Return')
                    self.observe(app, 'switch-ime-commit')

    def dialog_cycles(self, cycles):
        with self.app('egui'):
            self.engine('m17n:t:latn-post')
            for cycle in range(cycles):
                self.command('egui', focus='dialog')
                self.key('Return')
                time.sleep(0.5)
                opened = self.state('egui')
                assert opened['dialog'] and opened['dialog_text'] == ''
                assert opened['dialog_text_focused']
                for key in ['c', 'asciitilde']:
                    self.key(key)
                self.observe('egui', f'dialog-cycle-{cycle}-preedit')
                self.key('Return')
                committed = self.observe('egui', f'dialog-cycle-{cycle}-ime-commit')
                assert committed['dialog'] and committed['dialog_text'] == 'č', (
                    f"cycle {cycle}: expected composed č and open dialog after IME Enter; "
                    f"got text={committed['dialog_text']!r}, open={committed['dialog']}, "
                    f"save_count={committed['dialog_save']}")
                assert committed['dialog_save'] == cycle
                self.key('Return')
                closed = self.observe('egui', f'dialog-cycle-{cycle}-submit')
                assert not closed['dialog'] and closed['dialog_save'] == cycle + 1

    def gallery(self, binary):
        process = self.start('gallery', [binary, 'gtk', 'light'])
        try:
            window = wait(lambda: self.x('search', '--onlyvisible', '--name', '^egui_native_look gallery$').splitlines()[-1])
            self.x('windowfocus', '--sync', window)
            time.sleep(1)
            for _ in range(12):
                self.key('Tab')
            tree = self.tree('gallery', 'navigation')
            assert any((node.get('role') == 'frame' and node.get('name') == 'egui_native_look gallery' for node in tree))
            assert any((node.get('name') == 'Delay' and node.get('role') == 'spin button' for node in tree))
            assert any((node.get('name') == 'Volume' and node.get('role') == 'slider' for node in tree))
            assert any((node.get('name') == 'Search items' for node in tree))
            self.snapshot('gallery-orca')
            wait(lambda: 'egui_native_look gallery' in (self.out / 'orca-debug.log').read_text())
        finally:
            process.terminate()
            process.wait(timeout=5)

    def validate(self, apps):
        data = {(row['app'], row['case']): row['state'] for row in self.results}
        for app in apps:

            def state(case):
                return data[app, case]

            def dialog_open(case):
                return state(case).get('dialog_open', state(case).get('dialog'))
            assert state('entry-commit')['entry'] == 'č'
            assert state('entry-escape')['entry'] == 'čá'
            assert [state(c)['number'] for c in ['number-preedit', 'number-ime-commit', 'number-commit']] == [4, 4, 7]
            assert dialog_open('dialog-ime-commit') and state('dialog-ime-commit')['dialog_save'] == 0
            assert state('dialog-ime-commit')['dialog_text'] == 'č'
            assert not dialog_open('dialog-submit') and state('dialog-submit')['dialog_save'] == 1
            assert dialog_open('dialog-reopen') and state('dialog-reopen')['dialog_text'] == ''
            assert state('dialog-reopen')['dialog_text_focused']
            # GTK propagates the same Escape after m17n commits its preedit.
            # The project protects the dialog until a second Escape.
            assert dialog_open('dialog-first-escape') == (app == 'egui')
            assert state('dialog-first-escape')['dialog_text'] == 'á'
            assert not dialog_open('dialog-second-escape')
            assert state('dialog-first-escape')['dialog_save'] == state('dialog-second-escape')['dialog_save'] == 1
            assert state('atspi-number-set')['number'] == 6 and state('atspi-number-clamp')['number'] == 10
            assert state('atspi-slider-set')['slider'] == 75
            assert state('atspi-editable-text')['entry'] == 'č7'
            if app == 'egui':assert state('atspi-checkbox-action')['check']
            assert state('switch-focus-transfer')['widgets']['entry']['focus']
            assert state('switch-ime-commit')['entry'] == 'prefix|č'
            assert state('button-focus-transfer')['entry'] == 'prefix|č'
            if app == 'egui':
                assert state('atspi-disabled-number')['number'] == 4
        if 'egui' in apps:
            ime = (self.out / 'ime-events.jsonl').read_text()
            assert 'Preedit' in ime and 'Commit' in ime and ('č' in ime) and ('7' in ime)
        speech = [line for line in (self.out / 'orca-debug.log').read_text().splitlines() if 'SPEECH OUTPUT:' in line]
        assert any(('Number' in line and 'spin button' in line for line in speech))
        assert any(('Slider' in line and 'slider' in line for line in speech))
        assert any(('Name' in line for line in speech))
        if 'egui' in apps:
            assert any(('egui comparison' in line for line in speech))
        (self.out / 'announcements.txt').write_text('\n'.join(speech) + '\n')
        result = dict(apps=apps, observations=len(self.results), announcements=len(speech), status='passed', audio_playback=False)
        (self.out / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result), flush=True)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=pathlib.Path, required=True)
    parser.add_argument('--runtime', type=pathlib.Path, required=True)
    parser.add_argument('--ime-runtime', type=pathlib.Path, required=True)
    parser.add_argument('--binary', type=pathlib.Path, required=True)
    parser.add_argument('--apps', default='gtk,egui')
    parser.add_argument('--gallery-binary', type=pathlib.Path)
    parser.add_argument('--dialog-cycles', type=int, default=0)
    parser.add_argument('--dialog-cycles-only', action='store_true')
    args = parser.parse_args()
    live = Live(args.out.resolve(), args.runtime.resolve(), args.binary.resolve(), args.ime_runtime.resolve())
    try:
        with live.infrastructure():
            if args.dialog_cycles_only:
                if args.dialog_cycles < 1:
                    raise ValueError('--dialog-cycles-only requires positive --dialog-cycles')
                live.dialog_cycles(args.dialog_cycles)
                (live.out / 'verification.json').write_text(json.dumps(dict(
                    status='passed', dialog_cycles=args.dialog_cycles,
                    observations=len(live.results)), indent=2) + '\n')
            else:
                for app in args.apps.split(','):
                    live.exercise(app)
                    live.actions(app)
                if args.dialog_cycles:
                    live.dialog_cycles(args.dialog_cycles)
                if args.gallery_binary:
                    live.gallery(args.gallery_binary.resolve())
                live.validate(args.apps.split(','))
    except Exception as error:
        if live.started:
            (live.out / 'verification.json').write_text(json.dumps(dict(
                status='failed', observations=len(live.results), error=repr(error),
                last_case=live.results[-1]['case'] if live.results else None), indent=2) + '\n')
        raise
    print('Live tests completed', flush=True)
if __name__ == '__main__':
    main()
