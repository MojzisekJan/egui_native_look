from pathlib import Path
from collections import Counter
from PIL import Image
import json
import argparse

def measure(p):
    geometry = json.loads((p / 'geometry.json').read_text())
    names = ['button', 'check', 'switch', 'entry', 'number', 'slider', 'dropdown', 'progress', 'dialog', 'menu', 'plain', 'icon', 'icon_text', 'long', 'destructive', 'invalid']
    solid = ['button', 'plain', 'entry', 'number', 'destructive', 'invalid', 'switch']
    metrics = []
    for mode in ['light', 'dark']:
        for disabled in [False, True]:
            pair = {s['app']: s for s in geometry if s['mode'] == mode and s['disabled'] == disabled}
            data = dict(mode=mode, disabled=disabled, dimensions={}, solid_fills={})
            for name in names:
                sizes = {a: [s['widgets'][name].get('bounds_width', s['widgets'][name]['width']), s['widgets'][name]['height']] for a, s in pair.items()}
                data['dimensions'][name] = sizes
            for name in solid:
                colors = {}
                for app, state in pair.items():
                    w = state['widgets'][name]
                    tag = app + '-' + mode + ('-disabled' if disabled else '-normal')
                    with Image.open(p / (tag + '.png')) as im:
                        x, y = (int(w['x']), int(w['y']))
                        width = int(w.get('bounds_width', w['width']))
                        height = int(w['height'])
                        colors[app] = Counter(im.convert('RGB').crop((x + 4, y + 4, x + width - 4, y + height - 4)).getdata()).most_common(1)[0][0]
                colors['max_channel_difference'] = max((abs(a - b) for a, b in zip(colors['gtk'], colors['egui'])))
                data['solid_fills'][name] = colors
            metrics.append(data)
    (p / 'visual-metrics.json').write_text(json.dumps(metrics, indent=2) + '\n')
    print('Max channel difference:', max((v['max_channel_difference'] for m in metrics for v in m['solid_fills'].values())))
if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Measure control bounds and dominant interior colors from the GTK visual capture.')
    parser.add_argument('directory', type=Path)
    measure(parser.parse_args().directory)
