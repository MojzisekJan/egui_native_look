import SwiftUI
#if os(macOS)
import AppKit
#else
import UIKit
#endif

private let projects = ["Design studio", "Engineering", "Personal"]
private let priorities = ["High", "Medium", "Low"]
struct TaskItem: Identifiable, Codable { var id: UInt64; var title: String; var project: Int; var done: Bool; var priority: Int }
struct Document: Codable { var tasks: [TaskItem]; var notes: String; var minutes: Int; var dark: Bool; var look_index: Int }
@MainActor final class Workspace: ObservableObject {
    @Published var data: Document { didSet { save() } }
    @Published var page = 0
    @Published var filter = 0
    @Published var query = ""
    @Published var adding = false
    @Published var removing: UInt64?
    @Published var remaining: TimeInterval = 1500
    @Published var started: Date?
    @Published var message = "One thing at a time."
    @Published var saveError: String?
    var location: URL?
    var preview: Bool { location == nil }
    init() {
        let args = ProcessInfo.processInfo.arguments
        let seedURL = Bundle.main.url(forResource: "seed", withExtension: "json")!
        let seed = try! JSONDecoder().decode(Document.self, from: Data(contentsOf: seedURL))
        let preview = args.contains("--demo") || args.contains("--self-test")
        let folder = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].appendingPathComponent("Daybook", isDirectory: true)
        let file = folder.appendingPathComponent("swiftui.json")
        var value = seed
        var failure: String?
        if !preview && FileManager.default.fileExists(atPath: file.path) {
            do { value = try JSONDecoder().decode(Document.self, from: Data(contentsOf: file)) }
            catch { failure = "Could not read saved data: \(error.localizedDescription). Saving is disabled to protect it." }
        }
        value.minutes = min(60, max(5, value.minutes))
        for i in value.tasks.indices { value.tasks[i].project = min(2, max(0, value.tasks[i].project)); value.tasks[i].priority = min(2, max(0, value.tasks[i].priority)) }
        if args.contains("--dark") { value.dark = true }
        data = value; remaining = TimeInterval(value.minutes * 60)
        location = preview || failure != nil ? nil : file
        saveError = failure
        adding = args.contains("--show-add-task")
        removing = args.contains("--show-remove-task") ? data.tasks.first?.id : nil
    }
    func save() {
        guard let url = location else { return }
        do {
            try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            try JSONEncoder().encode(data).write(to: url, options: .atomic)
            saveError = nil
        } catch { saveError = "Changes could not be saved: \(error.localizedDescription)" }
    }
    @discardableResult func add(_ title: String, project: Int, priority: Int) -> Bool {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty else { return false }
        let id = (data.tasks.map(\.id).max() ?? 0) + 1
        data.tasks.append(TaskItem(id: id, title: title, project: min(2,max(0,project)), done: false, priority: min(2,max(0,priority))))
        filter = 0; query = ""; page = 1
        return true
    }
    var seconds: Int { max(0, Int(remaining - (started.map { Date().timeIntervalSince($0) } ?? 0))) }
    func toggleTimer() {
        if let start = started { remaining = max(0, remaining - Date().timeIntervalSince(start)); started = nil; message = "Take a breath. Resume when ready." }
        else { if remaining <= 0 { remaining = TimeInterval(data.minutes * 60) }; started = Date(); message = "You have this. Keep going." }
    }
    func resetTimer() { started = nil; remaining = TimeInterval(data.minutes * 60); message = "One thing at a time." }
    var completed: Int { data.tasks.filter(\.done).count }
    var title: String { switch page { case 0: "Make today count."; case 1: "Everything, in one place."; case 2: "A place for your ideas."; default: projects[min(2,max(0,page-3))] } }
    func visible(_ task: TaskItem) -> Bool { (page < 3 || task.project == page - 3) && (filter == 0 || (filter == 2) == task.done) && (query.isEmpty || task.title.localizedCaseInsensitiveContains(query)) }
}
private func color(_ r: Double, _ g: Double, _ b: Double) -> Color { Color(red: r/255, green: g/255, blue: b/255) }
struct Card<Content: View>: View {
    @Environment(\.colorScheme) var scheme
    var padding: CGFloat = 20
    @ViewBuilder var content: () -> Content
    var body: some View { content().frame(maxWidth: .infinity, alignment: .leading).padding(padding).background(scheme == .dark ? color(35,38,43) : .white, in: RoundedRectangle(cornerRadius: 14)).overlay(RoundedRectangle(cornerRadius: 14).stroke(scheme == .dark ? color(56,60,67) : color(229,232,236), lineWidth: 1)) }
}
struct Eyebrow: View { let text: String; var body: some View { Text(text).font(.system(size: 10, weight: .semibold)).foregroundStyle(.secondary) } }
struct Sidebar: View {
    @ObservedObject var model: Workspace
    @Environment(\.colorScheme) var scheme
    func nav(_ title: String, _ page: Int) -> some View {
        Button { model.page = page } label: { HStack { Text(title); Spacer() }.padding(.horizontal,10).padding(.vertical,8).background(model.page == page ? Color.accentColor.opacity(0.15) : .clear, in: RoundedRectangle(cornerRadius: 7)) }.buttonStyle(.plain).accessibilityAddTraits(model.page == page ? .isSelected : [])
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("daybook.").font(.system(size: 28,weight:.bold))
            Text("A little more intention.").font(.system(size:12)).foregroundStyle(.secondary).padding(.top,3)
            Eyebrow(text:"WORKSPACE").padding(.top,36).padding(.bottom,8)
            nav("My day",0); nav("All tasks",1); nav("Notes",2)
            Eyebrow(text:"PROJECTS").padding(.top,30).padding(.bottom,8)
            ForEach(0..<3) { nav(projects[$0],$0+3) }
            Divider().padding(.top,40).padding(.bottom,14)
            Text("Appearance").fontWeight(.semibold)
            #if os(macOS)
            Text("SwiftUI · macOS").font(.system(size:12)).foregroundStyle(.secondary).padding(.vertical,12)
            #else
            Text("SwiftUI · iOS").font(.system(size:12)).foregroundStyle(.secondary).padding(.vertical,12)
            #endif
            Toggle("Dark mode",isOn:$model.data.dark).toggleStyle(.switch)
            Text("Built with SwiftUI").font(.system(size:10)).foregroundStyle(.secondary).padding(.top,22)
            Spacer(minLength:0)
        }.padding(.horizontal,22).padding(.top,40).frame(width:210).frame(maxHeight:.infinity).background(scheme == .dark ? color(27,30,34) : color(239,242,246))
    }
}
struct FocusCard: View {
    @ObservedObject var model: Workspace
    var body: some View {
        TimelineView(.periodic(from:.now,by:0.25)) { _ in
            let seconds = model.seconds
            Card {
                VStack(alignment:.leading,spacing:12) {
                    Eyebrow(text:"TIME FOR FOCUS")
                    Text("Make room for deep work.").font(.system(size:15,weight:.semibold))
                    Text(String(format:"%02d:%02d",seconds/60,seconds%60)).font(.system(size:44)).monospacedDigit()
                    Text(model.message).font(.system(size:12)).foregroundStyle(.secondary)
                    HStack {
                        Button(model.started == nil ? "Start focus" : "Pause") { model.toggleTimer() }.buttonStyle(.borderedProminent)
                        Button("Reset") { model.resetTimer() }.buttonStyle(.bordered)
                    }.padding(.vertical,4)
                    ViewThatFits(in:.horizontal) {
                        HStack(spacing:8) { Text("Minutes").fixedSize(); minuteControls }
                        VStack(alignment:.leading,spacing:8) { Text("Minutes"); minuteControls }
                    }.disabled(model.started != nil)
                        .onChange(of:model.data.minutes) { _,_ in if model.started == nil { model.data.minutes = min(60,max(5,model.data.minutes)); model.resetTimer() } }
                }
            }.onChange(of:seconds) { _,value in
                if value == 0 && model.started != nil { model.started = nil; model.remaining = 0; model.message = "Session complete. Time for a break." }
            }
        }
    }
    var minuteControls: some View {
        HStack(spacing:8) {
            TextField("Minutes",value:$model.data.minutes,format:.number).textFieldStyle(.roundedBorder).frame(width:64).multilineTextAlignment(.center).accessibilityLabel("Focus minutes")
            Stepper("Minutes",value:$model.data.minutes,in:5...60).labelsHidden().fixedSize()
        }
    }

}
struct NotesCard: View {
    @ObservedObject var model: Workspace
    var large = false
    var body: some View {
        Card {
            VStack(alignment:.leading,spacing:12) {
                Text("Scratchpad").font(.system(size:18,weight:.semibold))
                Text("Ideas worth keeping.").font(.system(size:12)).foregroundStyle(.secondary)
                TextEditor(text:$model.data.notes).font(.system(size:13)).scrollContentBackground(.hidden).frame(minHeight:large ? 360 : 170).accessibilityLabel("Scratchpad")
                Text(model.preview ? "Preview session" : "Saved on this device").font(.system(size:10)).foregroundStyle(.secondary)
            }
        }
    }
}
struct TaskRow: View {
    @Binding var task: TaskItem
    var remove: () -> Void
    var body: some View {
        Card(padding:14) {
            HStack(spacing:12) {
                #if os(macOS)
                Toggle("Mark task complete",isOn:$task.done).labelsHidden().toggleStyle(.checkbox).accessibilityLabel(task.title)
                #else
                Button { task.done.toggle() } label: { Image(systemName:task.done ? "checkmark.circle.fill" : "circle").font(.system(size:23)).foregroundStyle(task.done ? Color.accentColor : .secondary) }.buttonStyle(.plain).accessibilityLabel(task.title).accessibilityValue(task.done ? "Completed" : "Active")
                #endif
                VStack(alignment:.leading,spacing:5) {
                    Text(task.title).font(.system(size:14)).strikethrough(task.done).foregroundStyle(task.done ? .secondary : .primary)
                    Text("\(Text(projects[task.project]).foregroundColor(.secondary))  · \(Text("\(priorities[task.priority]) priority").foregroundColor(task.priority == 0 ? color(192,103,52) : .secondary))").font(.system(size:11))
                }.frame(maxWidth:.infinity,alignment:.leading)
                Button("Remove",action:remove).buttonStyle(.bordered)
            }
        }
    }
}
struct AddTask: View {
    @ObservedObject var model: Workspace
    @Environment(\.dismiss) var dismiss
    @State var title = ""
    @State var project = 0
    @State var priority = 1
    @FocusState var editing: Bool
    private var fields: some View {
        VStack(alignment:.leading,spacing:16) {
            Text("Create a task").font(.title2.bold())
            Text("What would you like to move forward?").foregroundStyle(.secondary)
            TextField("Task title",text:$title).textFieldStyle(.roundedBorder).focused($editing)
            Picker("Project",selection:$project) { ForEach(0..<3) { Text(projects[$0]).tag($0) } }
            Picker("Priority",selection:$priority) { ForEach(0..<3) { Text(priorities[$0]).tag($0) } }.pickerStyle(.segmented)
        }
    }
    private var actions: some View {
        HStack { Spacer(); Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction).buttonStyle(.bordered); Button("Add task") { if model.add(title,project:project,priority:priority) { dismiss() } }.keyboardShortcut(.defaultAction).buttonStyle(.borderedProminent).disabled(title.trimmingCharacters(in:.whitespacesAndNewlines).isEmpty) }
    }
    var body: some View {
        Group {
#if os(iOS)
            // A single editor stays mounted while keyboard avoidance changes its height.
            GeometryReader { available in
                VStack(alignment:.leading,spacing:16) {
                    ScrollView { fields.frame(maxWidth:.infinity,alignment:.leading) }
                        .scrollBounceBehavior(.basedOnSize)
                        .frame(height:min(210,max(0,available.size.height-50)))
                    actions
                }.frame(maxWidth:.infinity,maxHeight:.infinity)
            }
#else
            VStack(alignment:.leading,spacing:16) { fields; actions }
#endif
        }.padding(28).frame(idealWidth:420).onAppear { editing = true; project = min(2,max(0,model.page-3)) }
    }
}
struct Content: View {
    @ObservedObject var model: Workspace
    var compact = false
    func metric(_ title: String, _ value: String) -> some View { Card(padding:compact ? 12 : 20) { VStack(alignment:.leading,spacing:8) { Eyebrow(text:title).lineLimit(1).minimumScaleFactor(0.8);Text(value).font(.system(size:compact ? 22 : 24,weight:.semibold)).lineLimit(1).minimumScaleFactor(0.8) } } }
    var body: some View {
        VStack(alignment:.leading,spacing:0) {
            Eyebrow(text:"YOUR PERSONAL WORKSPACE")
            Text(model.title).font(.system(size:30,weight:.bold)).padding(.top,8)
            Text("Small steps. Thoughtful work. A clearer day.").font(.system(size:13)).foregroundStyle(.secondary).padding(.top,5)
            HStack(spacing:8) { metric("TASKS DONE","\(model.completed) / \(model.data.tasks.count)");metric("FOCUS SESSION","\(model.data.minutes) min");metric("YOUR PROJECTS","3") }.padding(.top,26).padding(.bottom,24)
            if model.page == 2 { NotesCard(model:model,large:true) }
            else {
                HStack { Text("Your tasks").font(.system(size:20,weight:.semibold));Spacer();Button("+ New task") { model.adding = true }.buttonStyle(.borderedProminent) }
                ViewThatFits(in:.horizontal) {
                    HStack(spacing:12) { filters; search }
                    VStack(alignment:.leading,spacing:12) { filters;search }
                }.padding(.top,12).padding(.bottom,14)
                VStack(spacing:8) {
                    ForEach($model.data.tasks) { $task in
                        if model.visible(task) { TaskRow(task:$task) { model.removing = task.id } }
                    }
                    if !model.data.tasks.contains(where:model.visible) {
                        VStack(alignment:.leading,spacing:8) { Text("A clear page.").font(.title2);Text("No tasks match this view. Add a task or adjust your filters.").foregroundStyle(.secondary) }.frame(maxWidth:.infinity,alignment:.leading).padding(.vertical,25)
                    }
                }
                Text("\(model.completed) of \(model.data.tasks.count) completed — keep the momentum.").font(.system(size:11)).foregroundStyle(.secondary).padding(.top,20)
                if !model.data.tasks.isEmpty { ProgressView(value:Double(model.completed),total:Double(model.data.tasks.count)).padding(.top,8) }
            }
        }.frame(maxWidth:.infinity,alignment:.leading)
    }
    var filters: some View { Picker("Task filter",selection:$model.filter) { Text("All").tag(0);Text("Active").tag(1);Text("Done").tag(2) }.labelsHidden().pickerStyle(.segmented).frame(width:210) }
    var search: some View { TextField("Search tasks",text:$model.query).textFieldStyle(.roundedBorder).frame(width:180).accessibilityLabel("Search tasks") }
}
struct DaybookView: View {
    @StateObject var model = Workspace()
    @Environment(\.colorScheme) var scheme
    var body: some View {
        GeometryReader { geometry in
            let wide = geometry.size.width >= 980
            let sidebar = geometry.size.width >= 720
            HStack(spacing:0) {
                if sidebar { Sidebar(model:model) }
                ScrollView {
                    VStack(alignment:.leading,spacing:24) {
                        if !sidebar { HStack { Picker("Workspace",selection:$model.page) { Text("My day").tag(0);Text("All tasks").tag(1);Text("Notes").tag(2);ForEach(0..<3){Text(projects[$0]).tag($0+3)} };Toggle("Dark",isOn:$model.data.dark).toggleStyle(.switch) } }
                        Content(model:model,compact:geometry.size.width - (sidebar ? 210 : 0) - (wide ? 300 : 0) - 56 < 480)
                        if !wide { FocusCard(model:model);if model.page != 2 { NotesCard(model:model) } }
                        if let error = model.saveError { Text(error).foregroundStyle(.red) }
                    }.padding(28)
                }
                if wide { ScrollView { VStack(spacing:18) { FocusCard(model:model);NotesCard(model:model) }.padding(.horizontal,20).padding(.top,40).padding(.bottom,20) }.frame(width:300) }
            }.background(model.data.dark ? color(22,24,28) : color(248,249,251))
        }.preferredColorScheme(model.data.dark ? .dark : .light)
            .sheet(isPresented:$model.adding) { AddTask(model:model) }
            .alert(ProcessInfo.processInfo.arguments.contains("--long-alert") ? "Remove this task from your shared workspace?" : "Remove this task?",isPresented:Binding(get:{model.removing != nil},set:{if !$0 {model.removing = nil}})) {
                Button("Keep task",role:.cancel) { model.removing = nil }
                Button("Remove",role:.destructive) { let id=model.removing;model.data.tasks.removeAll{$0.id==id};model.removing=nil }
            } message: { Text(ProcessInfo.processInfo.arguments.contains("--long-alert") ? "This task and its notes will be removed from your workspace. Other tasks will stay exactly as they are." : "This task will be removed from your workspace.") }
            .onAppear { captureIfRequested() }
    }
    func captureIfRequested() {
        #if os(macOS)
        guard let path=ProcessInfo.processInfo.environment["DAYBOOK_SHOT"] else {return}
        DispatchQueue.main.asyncAfter(deadline:.now()+1) {
            if let window=NSApp.windows.first(where:{$0.isVisible}) {
                window.setContentSize(NSSize(width:1180,height:800))
                window.makeKeyAndOrderFront(nil)
                NSApp.activate(ignoringOtherApps:true)
            }
        }
        DispatchQueue.main.asyncAfter(deadline:.now()+3) {
            guard let window=NSApp.windows.first(where:{$0.isVisible}),let view=window.contentView else {fatalError("No capture window")}
            let bounds=view.convert(window.contentLayoutRect,from:nil)
            guard let bitmap=view.bitmapImageRepForCachingDisplay(in:bounds) else {fatalError("No capture bitmap")}
            view.cacheDisplay(in:bounds,to:bitmap)
            try! bitmap.representation(using:.png,properties:[:])!.write(to:URL(fileURLWithPath:path))
            NSApp.terminate(nil)
        }
        #else
        guard ProcessInfo.processInfo.arguments.contains("--capture-ios") else { return }
        DispatchQueue.main.asyncAfter(deadline:.now()+3) {
            guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                  let window = scene.windows.first(where: { $0.isKeyWindow }) else { return }
            let bounds = window.bounds.inset(by:window.safeAreaInsets)
            let format = UIGraphicsImageRendererFormat(); format.scale = 1
            let image = UIGraphicsImageRenderer(size:bounds.size,format:format).image { context in
                context.cgContext.translateBy(x:-bounds.minX,y:-bounds.minY)
                window.drawHierarchy(in:window.bounds,afterScreenUpdates:true)
            }
            let folder = FileManager.default.urls(for:.documentDirectory,in:.userDomainMask)[0]
            try! image.pngData()!.write(to:folder.appendingPathComponent("daybook.png"))
            let metadata: [String:Any] = ["width":bounds.width,"height":bounds.height,
                "safeTop":bounds.minY,"safeBottom":window.safeAreaInsets.bottom,
                "device":UIDevice.current.model,"system":UIDevice.current.systemVersion]
            try! JSONSerialization.data(withJSONObject:metadata,options:.prettyPrinted).write(to:folder.appendingPathComponent("daybook.json"))
        }
        #endif
    }
}
@main struct DaybookApplication: App {
    init() {
        if ProcessInfo.processInfo.arguments.contains("--self-test") {
            let model=Workspace();let count=model.data.tasks.count
            precondition(!model.add("  ",project:0,priority:1));precondition(model.add(" New task ",project:1,priority:0));precondition(model.data.tasks.count==count+1)
            precondition(model.data.tasks.last!.title=="New task")
            model.data.tasks[0].done=true;model.data.notes="Draft\nsecond line"
            let restored=try! JSONDecoder().decode(Document.self,from:JSONEncoder().encode(model.data))
            precondition(restored.tasks[0].done && restored.notes==model.data.notes)
            model.toggleTimer();precondition(model.started != nil);model.toggleTimer();precondition(model.started == nil);model.resetTimer();precondition(model.remaining==1500)
            print("Daybook SwiftUI model checks passed");exit(0)
        }
    }
    var body: some Scene {
        WindowGroup("Daybook · SwiftUI") {
            DaybookView()
            #if os(macOS)
                .frame(minWidth:390,minHeight:620)
            #endif
        }
        #if os(macOS)
        .defaultSize(width:1180,height:800)
        #endif
    }
}
