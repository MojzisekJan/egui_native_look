import gi, sys, json, pathlib, locale, os, time
locale.setlocale(locale.LC_ALL, '')
for name, version in [('Gtk', '4.0'), ('Adw', '1')]:
    gi.require_version(name, version)
from gi.repository import Gtk, Adw, GLib, Graphene, Gio
root = pathlib.Path(os.environ['GTK_PARITY_OUT'])
dark = 'dark' in sys.argv
pure = 'pure' in sys.argv
if not pure:
    Adw.init()
    Adw.StyleManager.get_default().set_color_scheme(Adw.ColorScheme.FORCE_DARK if dark else Adw.ColorScheme.FORCE_LIGHT)
else:
    Gtk.init()
settings = Gtk.Settings.get_default()
settings.set_property('gtk-font-name', 'DejaVu Sans 11')
settings.set_property('gtk-application-prefer-dark-theme', dark) if pure else None
win = Gtk.Window(title='GTK reference') if pure else Adw.Window(title='GTK reference')
win.set_default_size(int(os.getenv('TEST_WIDTH', '900')), int(os.getenv('TEST_HEIGHT', '900')))
win.set_decorated(False)
from gi.repository import Gdk
provider = Gtk.CssProvider()
provider.load_from_data(b'@define-color accent_bg_color #3584e4; @define-color accent_color #3584e4;')
Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
fixed = Gtk.Fixed()
win.set_child(fixed) if pure else win.set_content(fixed)
left_fixed = Gtk.Fixed()
right_fixed = Gtk.Fixed()
fixed.put(left_fixed, 0, 0)
fixed.put(right_fixed, 500, 0)
widgets = {}
counts = {'button': 0, 'dialog_save': 0}
dialog = None
dialog_entry = None
radio_selected = 0
list_selected = 0
list_activated = 0
heading = Gtk.Label(label=('GTK4' if pure else 'GTK4 + libadwaita') + ' — native reference')
fixed.put(heading, 20, 15)

def add(name, label, w, index):
    l = Gtk.Label(label=label, xalign=0)
    left_fixed.put(l, 20, 60 + 50 * index)
    w.set_size_request(240 if name not in ['switch', 'check'] else -1, -1)
    left_fixed.put(w, 180, 60 + 50 * index)
    widgets[name] = w
    if name in ['entry', 'number', 'slider', 'switch']:
        w.update_property([Gtk.AccessibleProperty.LABEL], [label])
b = Gtk.Button(label='Save')
b.add_css_class('suggested-action')
b.connect('clicked', lambda _: counts.update(button=counts['button'] + 1))
add('button', 'Button', b, 0)
c = Gtk.CheckButton(label='Notifications')
c.set_sensitive(not bool(os.getenv('DISABLED_CHECK')))
add('check', 'Checkbox', c, 1)
s = Gtk.Switch()
add('switch', 'Switch', s, 2)
e = Gtk.Entry()
e.set_text('Example')
add('entry', 'Text field', e, 3)
n = Gtk.SpinButton(adjustment=Gtk.Adjustment(value=4, lower=-10, upper=10, step_increment=1, page_increment=10), digits=2)
add('number', 'Number', n, 4)
a = Gtk.Adjustment(value=40, lower=-10, upper=100, step_increment=1, page_increment=10)
r = Gtk.Scale(orientation=Gtk.Orientation.HORIZONTAL, adjustment=a)
r.set_draw_value(False)
add('slider', 'Slider', r, 5)
d = Gtk.DropDown.new_from_strings(['Alpha', 'Beta', 'Gamma'])
add('dropdown', 'Dropdown', d, 6)
p = Gtk.ProgressBar()
p.set_fraction(0.4)
add('progress', 'Progress', p, 7)

def show_dialog(_):
    global dialog, dialog_entry
    print("DIALOG OPEN", time.monotonic(), flush=True)
    dialog = Adw.AlertDialog.new('Save changes?', 'Your preferences will be saved.')
    dialog.add_response('cancel', 'Cancel')
    dialog.add_response('save', 'Save')
    dialog.set_response_appearance('save', Adw.ResponseAppearance.SUGGESTED)
    dialog.set_default_response('save')
    dialog.set_close_response('cancel')
    if os.getenv('IME_DIALOG'):
        dialog_entry = Gtk.Entry()
        dialog_entry.set_activates_default(True)
        dialog_entry.update_property([Gtk.AccessibleProperty.LABEL], ['Name'])
        dialog.set_extra_child(dialog_entry)
        GLib.timeout_add(200, lambda: dialog_entry.grab_focus() and False)
    dialog.connect('response', lambda _, response: (print('DIALOG RESPONSE', response, time.monotonic(), flush=True), counts.update(dialog_save=counts['dialog_save'] + (response == 'save'))))
    dialog.present(win)
b = Gtk.Button(label='Open dialog')
b.connect('clicked', show_dialog)
add('dialog', 'Dialog', b, 8)
counts['menu_selected'] = 0
group = Gio.SimpleActionGroup()
for name, index in [('unavailable', -1), ('alpha', 1), ('beta', 2)]:
    action = Gio.SimpleAction.new(name, None)
    action.set_enabled(index != -1)
    action.connect('activate', lambda a, p, i=index: counts.update(menu_selected=i))
    group.add_action(action)
win.insert_action_group('demo', group)
model = Gio.Menu()
model.append('Unavailable', 'demo.unavailable')
sub = Gio.Menu()
sub.append('Alpha', 'demo.alpha')
sub.append('Beta', 'demo.beta')
model.append_submenu('More', sub)
menu = Gtk.MenuButton(label='Nested menu')
menu.set_popover(Gtk.PopoverMenu.new_from_model_full(model, Gtk.PopoverMenuFlags.NESTED))
add('menu', 'Nested menu', menu, 9)

def extra(name, w, index):
    right_fixed.put(w, 0, 60 + 60 * index)
    widgets[name] = w
extra('plain', Gtk.Button(label='Save'), 0)
extra('icon', Gtk.Button(icon_name='list-add-symbolic'), 1)
bb = Gtk.Button()
content = Gtk.Box(spacing=6)
content.append(Gtk.Image.new_from_icon_name('list-add-symbolic'))
content.append(Gtk.Label(label='Add item'))
bb.set_child(content)
extra('icon_text', bb, 2)
extra('long', Gtk.Button(label='Uložit všechny provedené změny'), 3)
spin = Adw.SpinRow(title='Count', adjustment=Gtk.Adjustment(value=4, lower=-10, upper=10, step_increment=1, page_increment=10), digits=2)
spin.set_size_request(320, -1)
extra('form', spin, 4)
lb = Gtk.ListBox()
lb.set_size_request(320, -1)
rr = Gtk.ListBoxRow()
rr.set_child(Gtk.Label(label='Documents', xalign=0))
rr.set_size_request(-1, 34)
lb.append(rr)
extra('list', lb, 5)
bb = Gtk.Button(label='Delete')
bb.add_css_class('destructive-action')
extra('destructive', bb, 6)
bb = Gtk.Entry()
bb.set_text('Example')
bb.add_css_class('error')
bb.set_size_request(240, -1)
extra('invalid', bb, 7)
outer_scroll = Gtk.ScrolledWindow()
outer_scroll.set_size_request(300, 180)
outer_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
outer_scroll.set_child(outer_box)
inner_scroll = Gtk.ScrolledWindow()
inner_scroll.set_size_request(-1, 90)
inner_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
inner_scroll.set_child(inner_box)
outer_box.append(inner_scroll)
for i in range(8):
    inner_box.append(Gtk.Button(label=f'Nested item {i}'))
for i in range(12):
    outer_box.append(Gtk.Button(label=f'Item {i}'))
fixed.put(outer_scroll, 500, 650)
radio_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
radio_buttons = []
for i, label in enumerate(['First', 'Second', 'Third']):
    rb = Gtk.CheckButton(label=label)
    if radio_buttons:
        rb.set_group(radio_buttons[0])
    radio_buttons.append(rb)
    radio_box.append(rb)
radio_buttons[0].set_active(True)
left_fixed.put(Gtk.Label(label='Radio group', xalign=0), 20, 580)
left_fixed.put(radio_box, 180, 580)
widgets['radio'] = radio_box
list_scroll = Gtk.ScrolledWindow()
list_scroll.set_size_request(240, 160)
listbox = Gtk.ListBox()
listbox.set_selection_mode(Gtk.SelectionMode.SINGLE)
listbox.set_activate_on_single_click(True)
for i in range(20):
    row = Gtk.ListBoxRow()
    row.set_child(Gtk.Label(label=f'File {i:02}', xalign=0))
    row.set_size_request(-1, 30)
    if i == 2:
        row.set_sensitive(False)
    listbox.append(row)
list_scroll.set_child(listbox)
listbox.select_row(listbox.get_row_at_index(0))

def activated(_, row):
    global list_activated
    list_activated += 1
listbox.connect('row-activated', activated)
left_fixed.put(Gtk.Label(label='List box', xalign=0), 20, 650)
left_fixed.put(list_scroll, 180, 650)
widgets['listbox'] = listbox
if os.getenv('STATE') == 'disabled':
    fixed.set_sensitive(False)
if os.getenv('LONG_DIALOG'):
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
    box.append(Gtk.Label(label='Test dlouhého dialogu'))
    win.set_content(box)

    def open_long():
        global dialog
        dialog = Adw.AlertDialog.new('Opravdu chcete uložit všechny neuložené změny v tomto dokumentu?', 'Před zavřením dokumentu můžete uložit změny, pokračovat v úpravách nebo změny zahodit. Tento delší český text musí zůstat celý čitelný i v úzkém okně.')
        for name, label in [('cancel', 'Pokračovat v úpravách dokumentu'), ('save', 'Uložit všechny provedené změny')]:
            dialog.add_response(name, label)
        dialog.set_default_response('save')
        dialog.set_close_response('cancel')
        dialog.present(win)
        return False
    GLib.timeout_add(500, open_long)
win.present()
last = None
sequence = 0
frame = 0

def poll():
    global last, radio_selected, sequence, frame
    frame += 1
    req = root / 'gtk-command.json'
    if req.exists():
        msg = json.loads(req.read_text())
        req.unlink()
        sequence = msg.get('seq', sequence)
        if msg.get('reset'):
            c.set_active(False)
            s.set_active(False)
            n.set_value(4)
            r.set_value(40)
            d.set_selected(0)
            e.set_text('Example')
            counts.update(button=0, dialog_save=0)
        if 'dark' in msg:
            Adw.StyleManager.get_default().set_color_scheme(Adw.ColorScheme.FORCE_DARK if msg['dark'] else Adw.ColorScheme.FORCE_LIGHT)
        if 'accent' in msg:
            from gi.repository import Gdk
            provider = Gtk.CssProvider()
            provider.load_from_data(('@define-color accent_bg_color #%06x; @define-color accent_color #%06x;' % (msg['accent'], msg['accent'])).encode())
            Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
        if 'password' in msg:
            e.set_visibility(not msg['password'])
        if 'controls_enabled' in msg:
            fixed.set_sensitive(msg['controls_enabled'])
        if 'text' in msg:
            e.set_text(msg['text'])
        if 'number' in msg:
            n.set_value(msg['number'])
        if 'dropdown' in msg:
            d.set_selected(msg['dropdown'])
        if 'slider' in msg:
            r.set_value(msg['slider'])
        if 'list_disabled' in msg:
            listbox.get_row_at_index(2).set_sensitive(not msg['list_disabled'])
        if 'list_selected' in msg:
            listbox.select_row(listbox.get_row_at_index(msg['list_selected']))
        if 'radio' in msg:
            radio_buttons[msg['radio']].set_active(True)
        if msg.get('focus'):
            name = msg['focus']
            if name == 'radio':
                radio_buttons[next((i for i, rb in enumerate(radio_buttons) if rb.get_active()))].grab_focus()
            elif name == 'listbox':
                listbox.get_selected_row().grab_focus()
            else:
                widgets[name].grab_focus()
        if 'number_text' in msg:
            n.set_text(msg['number_text'])
    radio_selected = next((i for i, rb in enumerate(radio_buttons) if rb.get_active()), -1)
    data = {'frame': frame, 'sequence': sequence, 'radio': radio_selected, 'list_selected': listbox.get_selected_row().get_index() if listbox.get_selected_row() else None, 'list_activated': list_activated, 'list_scroll': list_scroll.get_vadjustment().get_value(), 'number_update_policy': int(n.get_update_policy()), 'check': c.get_active(), 'switch': s.get_active(), 'number': n.get_value(), 'slider': r.get_value(), 'dropdown': d.get_selected(), 'entry': e.get_text(), 'number_text': n.get_text(), 'selection': e.get_selection_bounds(), 'cursor': e.get_position(), **counts}
    data['scroll'] = [outer_scroll.get_vadjustment().get_value(), inner_scroll.get_vadjustment().get_value()]
    data['widgets'] = {}
    focus = win.get_focus()
    for name, w in widgets.items():
        _, rect = w.compute_bounds(win)
        tx, ty = win.get_surface_transform()
        data['widgets'][name] = {'x': rect.get_x() + tx, 'y': rect.get_y() + ty, 'width': w.get_width(), 'height': rect.get_height(), 'bounds_width': rect.get_width(), 'focus': focus == w or (focus is not None and focus.is_ancestor(w))}
    data['dialog_text'] = dialog_entry.get_text() if dialog_entry else None
    data['dialog_text_focused'] = focus is not None and dialog_entry is not None and focus.is_ancestor(dialog_entry)
    data['dialog_open'] = dialog is not None and dialog.get_mapped()
    data['focus_visible'] = win.get_focus_visible()
    data['focus_type'] = type(focus).__name__ if focus else None
    data['gtk_version'] = [Gtk.get_major_version(), Gtk.get_minor_version(), Gtk.get_micro_version()]
    data['adwaita_version'] = [Adw.get_major_version(), Adw.get_minor_version(), Adw.get_micro_version()]
    data['dropdown_open'] = d.get_first_child().get_active()
    data['number_selection'] = n.get_selection_bounds()
    data['font'] = settings.get_property('gtk-font-name')
    data['theme'] = settings.get_property('gtk-theme-name')
    (root / 'gtk-state.json').write_text(json.dumps(data, indent=2))
    return True
GLib.timeout_add(50, poll)
loop = GLib.MainLoop()
win.connect('close-request', lambda _: loop.quit())
loop.run()
