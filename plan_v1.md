🎯 Complete Implementation Plan for Forge
✅ What You Have (Current State)
Working Components:

Rust Parser Engine (forge_engine)

✅ Dart AST parsing
✅ Widget tree extraction
✅ Static UI reconstruction
✅ PropValue system for properties
✅ Expression placeholder system


CLI Tool (forge_cli)

✅ Import command (Dart → JSON graph)
✅ Export command (JSON → Dart)
✅ Round-trip for static widgets


Test Coverage

✅ Static screens (complex layouts work)
✅ Stateful screens (partial - UI only, no behavior)



Current Capabilities:

Parse single .dart files
Extract widget hierarchy perfectly
Handle nested layouts (Scaffold, Column, Row, etc.)
Preserve styling (colors, padding, text styles)
Generate compilable Dart code

Current Limitations:

❌ Only single-file analysis
❌ StatefulWidget behavior lost (controllers, lifecycle)
❌ No state management detection (Provider, BLoC)
❌ No navigation flow analysis
❌ No conditional rendering logic
❌ No cross-file linking
❌ No visual studio yet


🎯 What You Need (Target State)
Vision: Full App Flow Visualization
Input: Flutter repo
Output: Interactive graph showing:
  - All screens
  - Navigation flows (screen A → screen B)
  - State dependencies (Provider X → Widget Y)
  - Conditional logic (if logged in → Dashboard, else → Login)
  - User interactions (button click → API call → navigate)

📋 COMPLETE STEP-BY-STEP PLAN

🔷 PHASE 1: STATEFUL WIDGET SUPPORT (2-3 weeks)
Goal: Handle StatefulWidget with setState() correctly
Step 1.1: Extend AST Analysis for State
File: forge_engine/src/lib.rs
rust// Add new types
pub struct StatefulAnalysis {
    pub state_class: String,           // "_CounterState"
    pub state_variables: Vec<StateVar>,
    pub lifecycle_methods: Vec<LifecycleMethod>,
    pub mutations: Vec<StateMutation>,
}

pub struct StateVar {
    pub name: String,        // "_count"
    pub var_type: String,    // "int"
    pub initial_value: Option<String>,
    pub is_late: bool,
}

pub struct LifecycleMethod {
    pub name: String,        // "initState", "dispose"
    pub body: String,        // Preserved as-is
}

pub struct StateMutation {
    pub method_name: String, // "_increment"
    pub updates: Vec<String>, // ["_count"]
    pub body: String,        // Full method body
}
Step 1.2: Create Stateful Parser
New file: forge_engine/src/stateful_parser.rs
rustuse syn::{Item, ItemImpl};

pub fn parse_stateful_widget(class: &ItemStruct) -> StatefulAnalysis {
    // 1. Find the State<T> class
    let state_class = find_state_class(class);
    
    // 2. Extract state variables
    let state_vars = extract_state_fields(state_class);
    
    // 3. Extract lifecycle methods
    let lifecycle = extract_lifecycle_methods(state_class);
    
    // 4. Find setState calls
    let mutations = find_state_mutations(state_class);
    
    StatefulAnalysis {
        state_class: state_class.name,
        state_variables: state_vars,
        lifecycle_methods: lifecycle,
        mutations,
    }
}

fn extract_state_fields(state_class: &ItemStruct) -> Vec<StateVar> {
    // Find fields like: int _count = 0;
    // Track: name, type, initial value, late keyword
}

fn extract_lifecycle_methods(state_class: &ItemStruct) -> Vec<LifecycleMethod> {
    // Find: initState(), dispose(), didUpdateWidget()
    // Preserve entire method body as string
}

fn find_state_mutations(state_class: &ItemStruct) -> Vec<StateMutation> {
    // Find methods containing setState(() { ... })
    // Track what variables are modified
}
Step 1.3: Extend Graph Schema
File: forge_spec/graph_schema.json
json{
  "type": "stateful_widget",
  "widget_class": "Counter",
  "state_class": "_CounterState",
  "state": {
    "variables": [
      {
        "name": "_count",
        "type": "int",
        "initial": "0"
      }
    ],
    "lifecycle": [
      {
        "method": "initState",
        "body": "super.initState();\n_controller = AnimationController(...);"
      }
    ],
    "mutations": [
      {
        "method": "_increment",
        "updates": ["_count"],
        "body": "setState(() { _count++; });"
      }
    ]
  },
  "build_method": {
    "widget_tree": { /* existing widget tree */ }
  }
}
Step 1.4: Update Code Generator
File: forge_engine/src/flutter_renderer.rs
rustpub fn render_stateful_widget(analysis: &StatefulAnalysis) -> String {
    let mut output = String::new();
    
    // 1. Render widget class
    output.push_str(&format!("class {} extends StatefulWidget {{\n", analysis.widget_class));
    output.push_str("  @override\n");
    output.push_str(&format!("  State<{}> createState() => {}();\n", 
        analysis.widget_class, analysis.state_class));
    output.push_str("}\n\n");
    
    // 2. Render state class
    output.push_str(&format!("class {} extends State<{}> {{\n", 
        analysis.state_class, analysis.widget_class));
    
    // 3. Render state variables
    for var in &analysis.state_variables {
        output.push_str(&format!("  {} {}{};\n",
            if var.is_late { "late" } else { "" },
            var.var_type,
            var.name
        ));
    }
    
    // 4. Render lifecycle methods (preserved as-is)
    for method in &analysis.lifecycle_methods {
        output.push_str(&format!("\n  @override\n  void {}() {{\n", method.name));
        output.push_str(&method.body);
        output.push_str("  }\n");
    }
    
    // 5. Render mutation methods
    for mutation in &analysis.mutations {
        output.push_str(&format!("\n  void {}() {{\n", mutation.method_name));
        output.push_str(&mutation.body);
        output.push_str("  }\n");
    }
    
    // 6. Render build method
    output.push_str("\n  @override\n  Widget build(BuildContext context) {\n");
    output.push_str(&render_widget_tree(&analysis.build_method.widget_tree));
    output.push_str("  }\n");
    
    output.push_str("}\n");
    output
}
Step 1.5: Test Cases
Create: packages/forge_engine/fixtures/stateful/
dart// counter.dart
class Counter extends StatefulWidget {
  @override
  State<Counter> createState() => _CounterState();
}

class _CounterState extends State<Counter> {
  int _count = 0;
  
  void _increment() {
    setState(() {
      _count++;
    });
  }
  
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('Count: $_count'),
        ElevatedButton(
          onPressed: _increment,
          child: Text('Increment'),
        ),
      ],
    );
  }
}
Test: Should preserve state, lifecycle, and mutations in round-trip
Step 1.6: CLI Integration
Test command:
bashforge_cli import --file fixtures/stateful/counter.dart --output counter_graph.json
forge_cli export --graph counter_graph.json --output counter_output.dart

# Verify:
diff fixtures/stateful/counter.dart counter_output.dart

🔷 PHASE 2: PROJECT-LEVEL ANALYSIS (3-4 weeks)
Goal: Analyze entire Flutter projects, not just single files
Step 2.1: Project Discovery
New file: forge_engine/src/project_analyzer.rs
rustuse std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ProjectAnalysis {
    pub root: PathBuf,
    pub screens: Vec<Screen>,
    pub widgets: Vec<CustomWidget>,
    pub providers: Vec<StateProvider>,
    pub routes: NavigationGraph,
    pub dependencies: DependencyGraph,
}

pub struct Screen {
    pub file_path: PathBuf,
    pub class_name: String,
    pub widget_tree: WidgetTree,
    pub imports: Vec<String>,
}

pub fn analyze_project(root: &Path) -> Result<ProjectAnalysis> {
    // 1. Find all .dart files
    let dart_files = discover_dart_files(root);
    
    // 2. Parse each file
    let mut screens = Vec::new();
    let mut widgets = Vec::new();
    
    for file in dart_files {
        let content = std::fs::read_to_string(&file)?;
        let parsed = parse_dart_file(&content)?;
        
        // Classify: is it a screen, widget, provider, etc?
        if is_screen(&parsed) {
            screens.push(extract_screen(&parsed, &file));
        } else if is_custom_widget(&parsed) {
            widgets.push(extract_widget(&parsed));
        }
    }
    
    // 3. Build cross-file references
    let dependencies = link_dependencies(&screens, &widgets);
    
    Ok(ProjectAnalysis {
        root: root.to_path_buf(),
        screens,
        widgets,
        providers: Vec::new(), // Phase 3
        routes: NavigationGraph::default(), // Phase 3
        dependencies,
    })
}

fn discover_dart_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("dart".as_ref()))
        .filter(|e| !e.path().to_str().unwrap().contains(".dart_tool"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_screen(parsed: &ParsedFile) -> bool {
    // Heuristic: ends with "Screen" or "Page"
    // Or: located in lib/screens/ or lib/pages/
    parsed.class_name.ends_with("Screen") 
        || parsed.class_name.ends_with("Page")
        || parsed.file_path.contains("/screens/")
}
Step 2.2: Import Resolution
Add to: forge_engine/src/project_analyzer.rs
rustpub struct Import {
    pub path: String,           // "package:app/widgets/button.dart"
    pub alias: Option<String>,  // "as btn"
    pub show: Vec<String>,      // Specific imports
}

pub fn resolve_imports(file: &ParsedFile, project: &ProjectAnalysis) -> Vec<ResolvedImport> {
    let mut resolved = Vec::new();
    
    for import in &file.imports {
        if import.path.starts_with("package:") {
            // Find in project
            if let Some(target) = find_file_in_project(&import.path, project) {
                resolved.push(ResolvedImport {
                    import: import.clone(),
                    target_file: target,
                    symbols: extract_exported_symbols(target),
                });
            }
        }
    }
    
    resolved
}
Step 2.3: Custom Widget Registry
New file: forge_engine/src/widget_registry.rs
rustpub struct WidgetRegistry {
    pub built_in: HashMap<String, BuiltInWidget>,
    pub custom: HashMap<String, CustomWidget>,
}

pub struct CustomWidget {
    pub name: String,
    pub file_path: PathBuf,
    pub constructor_params: Vec<Param>,
    pub widget_type: WidgetType, // Stateless, Stateful
    pub implementation: WidgetImplementation,
}

pub enum WidgetImplementation {
    FullyParsed(WidgetTree),     // We understand it
    BlackBox(String),             // Preserve as-is
}

pub fn build_widget_registry(project: &ProjectAnalysis) -> WidgetRegistry {
    let mut registry = WidgetRegistry::default();
    
    // Add all custom widgets from project
    for widget in &project.widgets {
        registry.custom.insert(
            widget.name.clone(),
            widget.clone()
        );
    }
    
    registry
}
Step 2.4: Update CLI for Projects
File: packages/forge_cli/bin/forge_cli.dart
dartvoid main(List<String> args) {
  final parser = ArgParser()
    ..addCommand('import')
    ..addCommand('analyze'); // NEW
    
  final results = parser.parse(args);
  
  switch (results.command?.name) {
    case 'import':
      handleImport(results.command!);
    case 'analyze':  // NEW
      handleAnalyze(results.command!);
  }
}

void handleAnalyze(ArgResults args) {
  final projectRoot = args['project'] as String? ?? '.';
  
  print('Analyzing project at: $projectRoot');
  
  // Call Rust engine
  final result = ForgeEngine.analyzeProject(projectRoot);
  
  print('Found:');
  print('  - ${result.screens.length} screens');
  print('  - ${result.widgets.length} custom widgets');
  print('  - ${result.providers.length} state providers');
  
  // Output full analysis
  File('forge_analysis.json').writeAsStringSync(
    jsonEncode(result.toJson())
  );
  
  print('\nFull analysis saved to: forge_analysis.json');
}
```

### Step 2.5: Test with Real Project
**Create:** `fixtures/sample_app/`
```
sample_app/
├── lib/
│   ├── main.dart
│   ├── screens/
│   │   ├── home_screen.dart
│   │   └── profile_screen.dart
│   └── widgets/
│       └── custom_button.dart
└── pubspec.yaml
Test command:
bashcd fixtures/sample_app
forge_cli analyze --project .

# Should output:
# Found:
#   - 2 screens
#   - 1 custom widgets
#   - 0 state providers (Phase 3)

🔷 PHASE 3: STATE MANAGEMENT DETECTION (3-4 weeks)
Goal: Detect Provider, Riverpod, BLoC patterns
Step 3.1: Provider Pattern Detection
New file: forge_engine/src/state_detector.rs
rustpub enum StateManagementType {
    Provider,
    Riverpod,
    BLoC,
    GetX,
    VanillaSetState,
}

pub struct StateProvider {
    pub name: String,
    pub provider_type: StateManagementType,
    pub file_path: PathBuf,
    pub state_variables: Vec<StateVar>,
    pub methods: Vec<ProviderMethod>,
    pub consumers: Vec<ConsumerLocation>,
}

pub fn detect_provider_pattern(file: &ParsedFile) -> Option<StateProvider> {
    // Look for class extending ChangeNotifier
    if extends_change_notifier(file) {
        return Some(StateProvider {
            name: file.class_name.clone(),
            provider_type: StateManagementType::Provider,
            state_variables: extract_state_vars(file),
            methods: extract_provider_methods(file),
            consumers: Vec::new(), // Fill later
        });
    }
    
    None
}

fn extends_change_notifier(file: &ParsedFile) -> bool {
    file.extends.as_ref()
        .map(|s| s.contains("ChangeNotifier"))
        .unwrap_or(false)
}

fn extract_provider_methods(file: &ParsedFile) -> Vec<ProviderMethod> {
    // Find methods that call notifyListeners()
    file.methods.iter()
        .filter(|m| m.body.contains("notifyListeners()"))
        .map(|m| ProviderMethod {
            name: m.name.clone(),
            params: m.params.clone(),
            updates: find_state_updates(&m.body),
        })
        .collect()
}
Step 3.2: Consumer Detection
Add to: forge_engine/src/state_detector.rs
rustpub struct ConsumerLocation {
    pub file_path: PathBuf,
    pub widget_id: String,
    pub consumer_type: ConsumerType,
    pub bindings: Vec<StateBinding>,
}

pub enum ConsumerType {
    Consumer,           // Consumer<T>
    ConsumerWidget,     // extends ConsumerWidget
    ProviderOf,         // Provider.of<T>(context)
    Watch,              // context.watch<T>()
    Read,               // context.read<T>()
}

pub fn find_consumers(
    provider: &StateProvider,
    project: &ProjectAnalysis
) -> Vec<ConsumerLocation> {
    let mut consumers = Vec::new();
    
    for screen in &project.screens {
        // Look for Consumer<ProviderName>
        if let Some(consumer) = find_consumer_widget(screen, &provider.name) {
            consumers.push(consumer);
        }
        
        // Look for Provider.of<ProviderName>
        if let Some(provider_of) = find_provider_of(screen, &provider.name) {
            consumers.push(provider_of);
        }
    }
    
    consumers
}
Step 3.3: State Flow Graph
New file: forge_engine/src/state_flow.rs
rustpub struct StateFlowGraph {
    pub providers: Vec<StateProvider>,
    pub flows: Vec<StateFlow>,
}

pub struct StateFlow {
    pub provider: String,
    pub state_var: String,
    pub mutation: String,      // Method that changes it
    pub consumers: Vec<WidgetRef>,
    pub triggers: Vec<EventRef>,
}

pub fn build_state_flow(project: &ProjectAnalysis) -> StateFlowGraph {
    let mut flows = Vec::new();
    
    for provider in &project.providers {
        for var in &provider.state_variables {
            // Find what mutations affect this variable
            let mutations = provider.methods.iter()
                .filter(|m| m.updates.contains(&var.name))
                .collect();
            
            // Find what widgets consume this variable
            let consumers = find_widget_consumers(provider, var, project);
            
            // Find what events trigger the mutations
            let triggers = find_event_triggers(mutations, project);
            
            flows.push(StateFlow {
                provider: provider.name.clone(),
                state_var: var.name.clone(),
                mutation: mutations.first().map(|m| m.name.clone()).unwrap_or_default(),
                consumers,
                triggers,
            });
        }
    }
    
    StateFlowGraph {
        providers: project.providers.clone(),
        flows,
    }
}
Step 3.4: Test Fixture
Create: fixtures/provider_app/
dart// providers/counter_provider.dart
class CounterProvider extends ChangeNotifier {
  int _count = 0;
  
  int get count => _count;
  
  void increment() {
    _count++;
    notifyListeners();
  }
}

// screens/counter_screen.dart
class CounterScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Consumer<CounterProvider>(
      builder: (context, counter, child) {
        return Column(
          children: [
            Text('Count: ${counter.count}'),
            ElevatedButton(
              onPressed: () => context.read<CounterProvider>().increment(),
              child: Text('Increment'),
            ),
          ],
        );
      },
    );
  }
}
Expected output:
json{
  "providers": [
    {
      "name": "CounterProvider",
      "type": "Provider",
      "state": [
        {"name": "_count", "type": "int"}
      ],
      "methods": [
        {
          "name": "increment",
          "updates": ["_count"]
        }
      ]
    }
  ],
  "flows": [
    {
      "provider": "CounterProvider",
      "state_var": "_count",
      "mutation": "increment",
      "consumers": ["counter_screen.text_1"],
      "triggers": ["counter_screen.button_1.onPressed"]
    }
  ]
}

🔷 PHASE 4: NAVIGATION FLOW ANALYSIS (2-3 weeks)
Goal: Extract screen-to-screen navigation
Step 4.1: Navigation Pattern Detection
New file: forge_engine/src/navigation_analyzer.rs
rustpub struct NavigationGraph {
    pub routes: HashMap<String, Route>,
    pub transitions: Vec<Transition>,
}

pub struct Route {
    pub name: String,
    pub path: Option<String>,    // For named routes
    pub screen: String,
    pub guards: Vec<RouteGuard>,
}

pub struct Transition {
    pub from_screen: String,
    pub to_screen: String,
    pub trigger: EventRef,       // What causes this navigation
    pub method: NavigationMethod,
}

pub enum NavigationMethod {
    Push,
    PushReplacement,
    PushNamed,
    Pop,
    PopUntil,
}

pub fn analyze_navigation(project: &ProjectAnalysis) -> NavigationGraph {
    let mut transitions = Vec::new();
    
    for screen in &project.screens {
        // Find Navigator.push/pushNamed calls
        let nav_calls = find_navigator_calls(screen);
        
        for call in nav_calls {
            transitions.push(Transition {
                from_screen: screen.class_name.clone(),
                to_screen: extract_target_screen(&call),
                trigger: find_trigger_event(&call, screen),
                method: classify_nav_method(&call),
            });
        }
    }
    
    NavigationGraph {
        routes: extract_route_table(project),
        transitions,
    }
}

fn find_navigator_calls(screen: &Screen) -> Vec<NavigatorCall> {
    // Search for patterns:
    // Navigator.push(context, MaterialPageRoute(...))
    // Navigator.pushNamed(context, '/profile')
    // Navigator.of(context).push(...)
}

fn extract_target_screen(call: &NavigatorCall) -> String {
    match call {
        NavigatorCall::Push { route } => {
            // Extract from MaterialPageRoute(builder: (_) => ProfileScreen())
            extract_screen_from_builder(route)
        }
        NavigatorCall::PushNamed { route_name } => {
            // Look up in route table
            route_name.clone()
        }
    }
}
Step 4.2: Route Table Extraction
Add to: forge_engine/src/navigation_analyzer.rs
rustfn extract_route_table(project: &ProjectAnalysis) -> HashMap<String, Route> {
    // Find MaterialApp(routes: {...})
    let main_file = project.screens.iter()
        .find(|s| s.file_path.ends_with("main.dart"))
        .expect("No main.dart found");
    
    // Look for routes map
    let routes_map = find_routes_declaration(main_file);
    
    // Parse:
    // '/': (context) => HomeScreen(),
    // '/profile': (context) => ProfileScreen(),
    
    parse_routes_map(routes_map)
}
Step 4.3: Conditional Navigation
Add to: forge_engine/src/navigation_analyzer.rs
rustpub struct ConditionalNavigation {
    pub condition: Condition,
    pub true_route: String,
    pub false_route: String,
}

pub fn find_conditional_navigation(screen: &Screen) -> Vec<ConditionalNavigation> {
    // Look for patterns like:
    // if (isLoggedIn) {
    //   Navigator.pushNamed(context, '/dashboard');
    // } else {
    //   Navigator.pushNamed(context, '/login');
    // }
}

🔷 PHASE 5: CONDITIONAL LOGIC EXTRACTION (2 weeks)
Goal: Track conditional rendering and logic branches
Step 5.1: Conditional Render Detection
New file: forge_engine/src/conditional_analyzer.rs
rustpub struct ConditionalRender {
    pub location: WidgetLocation,
    pub condition: Condition,
    pub branches: Vec<RenderBranch>,
}

pub struct Condition {
    pub expression: String,
    pub depends_on: Vec<StateRef>,
}

pub struct RenderBranch {
    pub predicate: String,      // "true", "false", or value
    pub widget: WidgetRef,
}

pub fn find_conditional_renders(screen: &Screen) -> Vec<ConditionalRender> {
    // Look for patterns:
    // if (condition) Widget1() else Widget2()
    // condition ? Widget1() : Widget2()
    // switch (value) { case A: Widget1(); case B: Widget2(); }
}
```

---

## 🔷 PHASE 6: FORGE STUDIO - VISUAL INTERFACE (4-6 weeks)

**Goal:** Build the visual editor to display and edit the flow graphs

### Step 6.1: Project Structure
**Create:** `packages/forge_studio/lib/`
```
lib/
├── main.dart
├── models/
│   ├── app_model.dart
│   ├── flow_graph.dart
│   └── project_state.dart
├── screens/
│   ├── project_overview_screen.dart
│   ├── flow_graph_screen.dart
│   └── widget_inspector_screen.dart
├── widgets/
│   ├── flow_canvas.dart
│   ├── screen_node.dart
│   ├── navigation_edge.dart
│   └── state_flow_edge.dart
└── services/
    ├── forge_engine_service.dart
    └── graph_layout_service.dart
Step 6.2: Load Project Analysis
File: packages/forge_studio/lib/services/forge_engine_service.dart
dartimport 'dart:convert';
import 'dart:io';

class ForgeEngineService {
  Future<ProjectAnalysis> loadProject(String path) async {
    // Call Rust CLI
    final result = await Process.run(
      'forge_cli',
      ['analyze', '--project', path, '--output', 'json'],
    );
    
    if (result.exitCode != 0) {
      throw Exception('Failed to analyze project: ${result.stderr}');
    }
    
    final json = jsonDecode(result.stdout);
    return ProjectAnalysis.fromJson(json);
  }
}

class ProjectAnalysis {
  final List<Screen> screens;
  final List<StateProvider> providers;
  final NavigationGraph navigation;
  final StateFlowGraph stateFlow;
  
  factory ProjectAnalysis.fromJson(Map<String, dynamic> json) {
    // Parse from Rust engine output
  }
}
Step 6.3: Flow Canvas Widget
File: packages/forge_studio/lib/widgets/flow_canvas.dart
dartimport 'package:flutter/material.dart';
import 'package:graphview/graphview.dart';

class FlowCanvas extends StatefulWidget {
  final ProjectAnalysis analysis;
  
  @override
  State<FlowCanvas> createState() => _FlowCanvasState();
}

class _FlowCanvasState extends State<FlowCanvas> {
  late Graph graph;
  
  @override
  void initState() {
    super.initState();
    graph = buildGraph();
  }
  
  Graph buildGraph() {
    final graph = Graph();
    
    // Add screen nodes
    for (final screen in widget.analysis.screens) {
      final node = Node.Id(screen.className);
      graph.addNode(node);
    }
    
    // Add navigation edges
    for (final transition in widget.analysis.navigation.transitions) {
      graph.addEdge(
        Node.Id(transition.fromScreen),
        Node.Id(transition.toScreen),
      );
    }
    
    return graph;
  }
  
  @override
  Widget build(BuildContext context) {
    return InteractiveViewer(
      constrained: false,
      boundaryMargin: EdgeInsets.all(100),
      minScale: 0.1,
      maxScale: 4.0,
      child: GraphView(
        graph: graph,
        algorithm: SugiyamaAlgorithm(
          builder: SugiyamaConfiguration(),
        ),
        builder: (Node node) {
          return ScreenNode(
            screenName: node.key!.value as String,
            onTap: () => _showScreenDetails(node),
          );
        },
      ),
    );
  }
  
  void _showScreenDetails(Node node) {
    // Show detailed view of screen
  }
}
Step 6.4: Screen Node Widget
File: packages/forge_studio/lib/widgets/screen_node.dart
dartclass ScreenNode extends StatelessWidget {
  final String screenName;
  final VoidCallback onTap;
  
  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Colors.white,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: Colors.blue, width: 2),
          boxShadow: [
            BoxShadow(
              color: Colors.black12,
              blurRadius: 4,
              offset: Offset(0, 2),
            ),
          ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.phone_android, size: 32, color: Colors.blue),
            SizedBox(height: 8),
            Text(
              screenName,
              style: TextStyle(
                fontWeight: FontWeight.bold,
                fontSize: 14,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
Step 6.5: Main Application
File: packages/forge_studio/lib/main.dart
dartvoid main() {
  runApp(ForgeStudioApp());
}

class ForgeStudioApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Forge Studio',
      theme: ThemeData.dark(),
      home: ProjectOverviewScreen(),
    );
  }
}

class ProjectOver
dartclass ProjectOverviewScreen extends StatefulWidget {
  @override
  State<ProjectOverviewScreen> createState() => _ProjectOverviewScreenState();
}

class _ProjectOverviewScreenState extends State<ProjectOverviewScreen> {
  ProjectAnalysis? _analysis;
  bool _isLoading = false;
  String? _error;
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Forge Studio'),
        actions: [
          IconButton(
            icon: Icon(Icons.folder_open),
            onPressed: _openProject,
            tooltip: 'Open Project',
          ),
        ],
      ),
      body: _buildBody(),
    );
  }
  
  Widget _buildBody() {
    if (_isLoading) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Analyzing project...'),
          ],
        ),
      );
    }
    
    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.error, size: 64, color: Colors.red),
            SizedBox(height: 16),
            Text('Error: $_error'),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _openProject,
              child: Text('Try Again'),
            ),
          ],
        ),
      );
    }
    
    if (_analysis == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.folder_open, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text('No project loaded'),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _openProject,
              child: Text('Open Project'),
            ),
          ],
        ),
      );
    }
    
    return _buildProjectView();
  }
  
  Widget _buildProjectView() {
    return Row(
      children: [
        // Left sidebar - Project stats
        Container(
          width: 250,
          color: Colors.grey[900],
          child: _buildSidebar(),
        ),
        
        // Main content - Flow graph
        Expanded(
          child: _buildMainContent(),
        ),
      ],
    );
  }
  
  Widget _buildSidebar() {
    return ListView(
      padding: EdgeInsets.all(16),
      children: [
        Text(
          'Project Overview',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        SizedBox(height: 16),
        
        _buildStatCard(
          'Screens',
          _analysis!.screens.length,
          Icons.phone_android,
          Colors.blue,
        ),
        
        _buildStatCard(
          'Widgets',
          _analysis!.widgets.length,
          Icons.widgets,
          Colors.green,
        ),
        
        _buildStatCard(
          'Providers',
          _analysis!.providers.length,
          Icons.storage,
          Colors.orange,
        ),
        
        _buildStatCard(
          'Routes',
          _analysis!.navigation.transitions.length,
          Icons.route,
          Colors.purple,
        ),
        
        SizedBox(height: 24),
        Divider(),
        SizedBox(height: 8),
        
        ListTile(
          leading: Icon(Icons.account_tree),
          title: Text('Flow View'),
          selected: true,
          onTap: () {},
        ),
        
        ListTile(
          leading: Icon(Icons.view_list),
          title: Text('Screen List'),
          onTap: () {},
        ),
        
        ListTile(
          leading: Icon(Icons.bubble_chart),
          title: Text('State Graph'),
          onTap: () {},
        ),
      ],
    );
  }
  
  Widget _buildStatCard(String label, int count, IconData icon, Color color) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(icon, color: color, size: 32),
            SizedBox(width: 16),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '$count',
                  style: TextStyle(
                    fontSize: 24,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(label, style: TextStyle(color: Colors.grey)),
              ],
            ),
          ],
        ),
      ),
    );
  }
  
  Widget _buildMainContent() {
    return Column(
      children: [
        // Toolbar
        Container(
          height: 60,
          padding: EdgeInsets.symmetric(horizontal: 16),
          decoration: BoxDecoration(
            color: Colors.grey[850],
            border: Border(bottom: BorderSide(color: Colors.grey[700]!)),
          ),
          child: Row(
            children: [
              Text(
                'Application Flow',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
              ),
              Spacer(),
              IconButton(
                icon: Icon(Icons.zoom_in),
                onPressed: () {},
                tooltip: 'Zoom In',
              ),
              IconButton(
                icon: Icon(Icons.zoom_out),
                onPressed: () {},
                tooltip: 'Zoom Out',
              ),
              IconButton(
                icon: Icon(Icons.fit_screen),
                onPressed: () {},
                tooltip: 'Fit to Screen',
              ),
            ],
          ),
        ),
        
        // Canvas
        Expanded(
          child: FlowCanvas(analysis: _analysis!),
        ),
      ],
    );
  }
  
  Future<void> _openProject() async {
    // For now, use file picker or hardcoded path
    // In production, integrate with file_picker package
    
    setState(() {
      _isLoading = true;
      _error = null;
    });
    
    try {
      final service = ForgeEngineService();
      final analysis = await service.loadProject('./fixtures/sample_app');
      
      setState(() {
        _analysis = analysis;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }
}
Step 6.6: State Flow Visualization
File: packages/forge_studio/lib/widgets/state_flow_canvas.dart
dartclass StateFlowCanvas extends StatelessWidget {
  final StateFlowGraph stateFlow;
  
  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: StateFlowPainter(stateFlow),
      child: Container(),
    );
  }
}

class StateFlowPainter extends CustomPainter {
  final StateFlowGraph stateFlow;
  
  StateFlowPainter(this.stateFlow);
  
  @override
  void paint(Canvas canvas, Size size) {
    // Draw providers as boxes
    for (int i = 0; i < stateFlow.providers.length; i++) {
      final provider = stateFlow.providers[i];
      final rect = Rect.fromLTWH(50, 50 + i * 150, 200, 100);
      
      // Draw provider box
      final paint = Paint()
        ..color = Colors.orange
        ..style = PaintingStyle.fill;
      canvas.drawRRect(
        RRect.fromRectAndRadius(rect, Radius.circular(8)),
        paint,
      );
      
      // Draw provider name
      final textPainter = TextPainter(
        text: TextSpan(
          text: provider.name,
          style: TextStyle(color: Colors.white, fontSize: 14),
        ),
        textDirection: TextDirection.ltr,
      );
      textPainter.layout();
      textPainter.paint(
        canvas,
        Offset(rect.left + 10, rect.top + 10),
      );
    }
    
    // Draw flows as arrows from providers to widgets
    for (final flow in stateFlow.flows) {
      // Draw arrow from provider to each consumer
      for (final consumer in flow.consumers) {
        _drawArrow(canvas, /* provider position */, /* consumer position */);
      }
    }
  }
  
  void _drawArrow(Canvas canvas, Offset start, Offset end) {
    final paint = Paint()
      ..color = Colors.blue
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;
    
    canvas.drawLine(start, end, paint);
    
    // Draw arrowhead
    final arrowSize = 10.0;
    final angle = (end - start).direction;
    
    final arrowPath = Path()
      ..moveTo(end.dx, end.dy)
      ..lineTo(
        end.dx - arrowSize * cos(angle - pi / 6),
        end.dy - arrowSize * sin(angle - pi / 6),
      )
      ..lineTo(
        end.dx - arrowSize * cos(angle + pi / 6),
        end.dy - arrowSize * sin(angle + pi / 6),
      )
      ..close();
    
    canvas.drawPath(arrowPath, paint..style = PaintingStyle.fill);
  }
  
  @override
  bool shouldRepaint(StateFlowPainter oldDelegate) {
    return stateFlow != oldDelegate.stateFlow;
  }
}

🔷 PHASE 7: INTERACTIVE EDITING (4-6 weeks)
Goal: Allow users to modify the flow graph and regenerate code
Step 7.1: Graph Mutation API
File: forge_engine/src/graph_mutator.rs
rustpub struct GraphMutator {
    project: ProjectAnalysis,
}

impl GraphMutator {
    pub fn add_navigation(&mut self, from: &str, to: &str, trigger: &str) -> Result<()> {
        // 1. Find the source screen
        let screen = self.project.screens.iter_mut()
            .find(|s| s.class_name == from)
            .ok_or("Screen not found")?;
        
        // 2. Add navigation call to trigger location
        let nav_code = format!(
            "Navigator.push(context, MaterialPageRoute(builder: (_) => {}()))",
            to
        );
        
        // 3. Insert into widget tree at trigger point
        self.insert_callback(screen, trigger, &nav_code)?;
        
        // 4. Update navigation graph
        self.project.navigation.transitions.push(Transition {
            from_screen: from.to_string(),
            to_screen: to.to_string(),
            trigger: trigger.to_string(),
            method: NavigationMethod::Push,
        });
        
        Ok(())
    }
    
    pub fn add_widget(&mut self, screen: &str, parent: &str, widget: Widget) -> Result<()> {
        // Add widget to tree
    }
    
    pub fn modify_property(&mut self, widget_id: &str, prop: &str, value: PropValue) -> Result<()> {
        // Modify widget property
    }
    
    pub fn add_state_binding(&mut self, widget_id: &str, provider: &str, field: &str) -> Result<()> {
        // Connect widget to state provider
    }
}
Step 7.2: Studio Integration
File: packages/forge_studio/lib/services/graph_editor_service.dart
dartclass GraphEditorService {
  final ForgeEngineService _engine;
  ProjectAnalysis _analysis;
  
  GraphEditorService(this._engine, this._analysis);
  
  Future<void> addNavigation({
    required String fromScreen,
    required String toScreen,
    required String trigger,
  }) async {
    // Call Rust mutator
    final result = await Process.run('forge_cli', [
      'mutate',
      '--add-navigation',
      '--from', fromScreen,
      '--to', toScreen,
      '--trigger', trigger,
    ]);
    
    if (result.exitCode != 0) {
      throw Exception('Failed to add navigation: ${result.stderr}');
    }
    
    // Reload analysis
    _analysis = await _engine.loadProject(_analysis.rootPath);
  }
  
  Future<void> addWidget({
    required String screenName,
    required String parentId,
    required Widget widget,
  }) async {
    // Similar pattern
  }
  
  Future<void> regenerateCode() async {
    // Export entire project back to Dart
    final result = await Process.run('forge_cli', [
      'export',
      '--project', _analysis.rootPath,
    ]);
    
    if (result.exitCode != 0) {
      throw Exception('Failed to regenerate code: ${result.stderr}');
    }
  }
}
Step 7.3: Property Inspector Panel
File: packages/forge_studio/lib/widgets/property_inspector.dart
dartclass PropertyInspector extends StatelessWidget {
  final Widget? selectedWidget;
  final Function(String prop, dynamic value) onPropertyChanged;
  
  @override
  Widget build(BuildContext context) {
    if (selectedWidget == null) {
      return Center(child: Text('No widget selected'));
    }
    
    return ListView(
      padding: EdgeInsets.all(16),
      children: [
        Text(
          selectedWidget!.type,
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        SizedBox(height: 16),
        
        // Render property editors based on widget type
        ...selectedWidget!.properties.entries.map((entry) {
          return _buildPropertyEditor(entry.key, entry.value);
        }),
        
        SizedBox(height: 24),
        ElevatedButton(
          onPressed: () => _addProperty(),
          child: Text('Add Property'),
        ),
      ],
    );
  }
  
  Widget _buildPropertyEditor(String name, PropValue value) {
    return Padding(
      padding: EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(name, style: TextStyle(fontWeight: FontWeight.bold)),
          SizedBox(height: 8),
          _buildValueEditor(name, value),
        ],
      ),
    );
  }
  
  Widget _buildValueEditor(String name, PropValue value) {
    if (value is StringValue) {
      return TextField(
        controller: TextEditingController(text: value.value),
        onChanged: (newValue) => onPropertyChanged(name, newValue),
        decoration: InputDecoration(
          border: OutlineInputBorder(),
          hintText: 'Enter $name',
        ),
      );
    } else if (value is NumberValue) {
      return TextField(
        controller: TextEditingController(text: value.value.toString()),
        keyboardType: TextInputType.number,
        onChanged: (newValue) => onPropertyChanged(name, double.tryParse(newValue)),
        decoration: InputDecoration(
          border: OutlineInputBorder(),
          hintText: 'Enter $name',
        ),
      );
    } else if (value is ColorValue) {
      return GestureDetector(
        onTap: () => _pickColor(name, value),
        child: Container(
          height: 40,
          decoration: BoxDecoration(
            color: value.color,
            border: Border.all(color: Colors.grey),
            borderRadius: BorderRadius.circular(4),
          ),
        ),
      );
    }
    
    // Fallback for complex types
    return Text(value.toString());
  }
  
  void _pickColor(String name, ColorValue current) {
    // Show color picker
  }
  
  void _addProperty() {
    // Show dialog to add new property
  }
}

🔷 PHASE 8: CODE GENERATION & EXPORT (2-3 weeks)
Goal: Generate clean, production-ready Flutter code from modified graphs
Step 8.1: Full Project Export
File: forge_engine/src/project_exporter.rs
rustpub struct ProjectExporter {
    analysis: ProjectAnalysis,
    output_dir: PathBuf,
}

impl ProjectExporter {
    pub fn export_full_project(&self) -> Result<()> {
        // 1. Export all screens
        for screen in &self.analysis.screens {
            self.export_screen(screen)?;
        }
        
        // 2. Export all custom widgets
        for widget in &self.analysis.widgets {
            self.export_widget(widget)?;
        }
        
        // 3. Export state providers
        for provider in &self.analysis.providers {
            self.export_provider(provider)?;
        }
        
        // 4. Export main.dart with routes
        self.export_main()?;
        
        // 5. Format all files
        self.format_dart_files()?;
        
        Ok(())
    }
    
    fn export_screen(&self, screen: &Screen) -> Result<()> {
        let code = self.render_screen(screen)?;
        let output_path = self.output_dir.join(&screen.file_path);
        
        // Ensure directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(output_path, code)?;
        Ok(())
    }
    
    fn render_screen(&self, screen: &Screen) -> Result<String> {
        let mut output = String::new();
        
        // 1. Imports
        output.push_str("import 'package:flutter/material.dart';\n");
        for import in &screen.imports {
            output.push_str(&format!("import '{}';\n", import));
        }
        output.push_str("\n");
        
        // 2. Class declaration
        if screen.is_stateful {
            output.push_str(&self.render_stateful_widget(screen)?);
        } else {
            output.push_str(&self.render_stateless_widget(screen)?);
        }
        
        Ok(output)
    }
    
    fn export_main(&self) -> Result<()> {
        let mut output = String::new();
        
        output.push_str("import 'package:flutter/material.dart';\n\n");
        
        // Import all screens
        for screen in &self.analysis.screens {
            let import_path = screen.file_path.strip_prefix("lib/")
                .unwrap_or(&screen.file_path);
            output.push_str(&format!("import '{}';\n", import_path.display()));
        }
        
        output.push_str("\nvoid main() {\n");
        output.push_str("  runApp(MyApp());\n");
        output.push_str("}\n\n");
        
        output.push_str("class MyApp extends StatelessWidget {\n");
        output.push_str("  @override\n");
        output.push_str("  Widget build(BuildContext context) {\n");
        output.push_str("    return MaterialApp(\n");
        output.push_str("      title: 'Flutter App',\n");
        output.push_str("      theme: ThemeData(primarySwatch: Colors.blue),\n");
        
        // Generate routes
        if !self.analysis.navigation.routes.is_empty() {
            output.push_str("      routes: {\n");
            for (path, route) in &self.analysis.navigation.routes {
                output.push_str(&format!(
                    "        '{}': (context) => {}(),\n",
                    path, route.screen
                ));
            }
            output.push_str("      },\n");
        }
        
        // Set initial route
        if let Some(first_screen) = self.analysis.screens.first() {
            output.push_str(&format!("      home: {}(),\n", first_screen.class_name));
        }
        
        output.push_str("    );\n");
        output.push_str("  }\n");
        output.push_str("}\n");
        
        std::fs::write(self.output_dir.join("lib/main.dart"), output)?;
        Ok(())
    }
    
    fn format_dart_files(&self) -> Result<()> {
        // Run dart format on output directory
        std::process::Command::new("dart")
            .arg("format")
            .arg(&self.output_dir)
            .output()?;
        
        Ok(())
    }
}
Step 8.2: CLI Export Command
File: packages/forge_cli/bin/forge_cli.dart
dartvoid handleExport(ArgResults args) {
  final analysisFile = args['analysis'] as String? ?? 'forge_analysis.json';
  final outputDir = args['output'] as String? ?? './output';
  
  print('Exporting project to: $outputDir');
  
  // Call Rust exporter
  final result = ForgeEngine.exportProject(
    analysisFile: analysisFile,
    outputDir: outputDir,
  );
  
  if (result.success) {
    print('✅ Project exported successfully!');
    print('Files generated:');
    for (final file in result.files) {
      print('  - $file');
    }
  } else {
    print('❌ Export failed: ${result.error}');
    exit(1);
  }
}

🔷 PHASE 9: ADVANCED FEATURES (4-6 weeks)
Goal: Polish and advanced capabilities
Step 9.1: Diff & Merge System
File: forge_engine/src/diff_engine.rs
rustpub struct ProjectDiff {
    pub added_screens: Vec<Screen>,
    pub removed_screens: Vec<Screen>,
    pub modified_screens: Vec<ScreenDiff>,
    pub navigation_changes: Vec<NavigationChange>,
}

pub struct ScreenDiff {
    pub screen_name: String,
    pub widget_changes: Vec<WidgetChange>,
    pub property_changes: Vec<PropertyChange>,
}

pub fn compute_diff(original: &ProjectAnalysis, modified: &ProjectAnalysis) -> ProjectDiff {
    // Compare two versions and generate diff
}

pub fn apply_diff(project: &mut ProjectAnalysis, diff: &ProjectDiff) -> Result<()> {
    // Apply changes from diff
}

pub fn merge_changes(
    base: &ProjectAnalysis,
    ours: &ProjectAnalysis,
    theirs: &ProjectAnalysis,
) -> Result<ProjectAnalysis> {
    // Three-way merge for collaboration
}
Step 9.2: Hot Reload Integration
File: forge_engine/src/hot_reload.rs
rustpub struct HotReloadService {
    device_id: String,
}

impl HotReloadService {
    pub fn send_update(&self, widget_id: &str, new_props: &HashMap<String, PropValue>) -> Result<()> {
        // Send incremental update to running Flutter app
        // Uses Flutter DevTools protocol
    }
    
    pub fn full_reload(&self, analysis: &ProjectAnalysis) -> Result<()> {
        // Trigger full hot reload
    }
}
Step 9.3: Component Library
File: packages/forge_studio/lib/widgets/component_palette.dart
dartclass ComponentPalette extends StatelessWidget {
  final Function(Widget widget) onWidgetDragged;
  
  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        _buildCategory('Layout', [
          _buildComponentTile('Container', Icons.crop_square),
          _buildComponentTile('Row', Icons.table_rows),
          _buildComponentTile('Column', Icons.view_column),
          _buildComponentTile('Stack', Icons.layers),
        ]),
        
        _buildCategory('Input', [
          _buildComponentTile('TextField', Icons.text_fields),
          _buildComponentTile('Button', Icons.smart_button),
          _buildComponentTile('Checkbox', Icons.check_box),
          _buildComponentTile('Switch', Icons.toggle_on),
        ]),
        
        _buildCategory('Display', [
          _buildComponentTile('Text', Icons.text_format),
          _buildComponentTile('Image', Icons.image),
          _buildComponentTile('Icon', Icons.star),
          _buildComponentTile('Card', Icons.credit_card),
        ]),
      ],
    );
  }
  
  Widget _buildCategory(String name, List<Widget> children) {
    return ExpansionTile(
      title: Text(name),
      initiallyExpanded: true,
      children: children,
    );
  }
  
  Widget _buildComponentTile(String name, IconData icon) {
    return Draggable<String>(
      data: name,
      feedback: Material(
        elevation: 4,
        child: Container(
          padding: EdgeInsets.all(8),
          color: Colors.blue,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, color: Colors.white),
              SizedBox(width: 8),
              Text(name, style: TextStyle(color: Colors.white)),
            ],
          ),
        ),
      ),
      child: ListTile(
        leading: Icon(icon),
        title: Text(name),
        dense: true,
      ),
    );
  }
}
Step 9.4: Code Preview Panel
File: packages/forge_studio/lib/widgets/code_preview.dart
dartimport 'package:flutter_highlight/flutter_highlight.dart';
import 'package:flutter_highlight/themes/github.dart';

class CodePreview extends StatelessWidget {
  final String dartCode;
  
  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.white,
      child: SingleChildScrollView(
        child: HighlightView(
          dartCode,
          language: 'dart',
          theme: githubTheme,
          padding: EdgeInsets.all(16),
          textStyle: TextStyle(fontSize: 14, fontFamily: 'monospace'),
        ),
      ),
    );
  }
}
```

---

## 🔷 PHASE 10: POLISH & RELEASE (2-4 weeks)

**Goal:** Production readiness

### Step 10.1: Error Handling
- Graceful failure for unparseable code
- Clear error messages with suggestions
- Rollback mechanism for failed exports

### Step 10.2: Performance Optimization
- Lazy loading for large projects
- Incremental analysis (only changed files)
- Canvas virtualization for large graphs
- Background worker threads

### Step 10.3: Documentation
```
docs/
├── getting-started.md
├── architecture.md
├── api-reference.md
├── examples/
│   ├── simple-todo-app.md
│   ├── ecommerce-flow.md
│   └── social-media-app.md
└── troubleshooting.md
```

### Step 10.4: Testing
```
tests/
├── unit/
│   ├── parser_test.rs
│   ├── stateful_test.rs
│   └── navigation_test.rs
├── integration/
│   ├── full_project_test.rs
│   └── round_trip_test.rs
└── fixtures/
    ├── simple_app/
    ├── complex_app/
    └── real_world_app/
Step 10.5: CI/CD Pipeline
File: .github/workflows/ci.yml
yamlname: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Flutter
        uses: subosito/flutter-action@v2
        with:
          flutter-version: '3.16.0'
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Rust tests
        run: cargo test --all
      
      - name: Run Dart tests
        run: |
          cd packages/forge_cli
          dart test
      
      - name: Build engine
        run: cargo build --release
      
      - name: Integration tests
        run: ./scripts/run_integration_tests.sh

📊 TIMELINE SUMMARY
PhaseDurationMilestonePhase 1: StatefulWidget Support2-3 weeks✅ Handle state, lifecycle, setState()Phase 2: Project-Level Analysis3-4 weeks✅ Multi-file, imports, custom widgetsPhase 3: State Management3-4 weeks✅ Provider, Riverpod, BLoC detectionPhase 4: Navigation Flow2-3 weeks✅ Route extraction, screen transitionsPhase 5: Conditional Logic2 weeks✅ if/else, ternary, switch renderingPhase 6: Forge Studio UI4-6 weeks✅ Visual graph editorPhase 7: Interactive Editing4-6 weeks✅ Modify graph, regenerate codePhase 8: Code Export2-3 weeks✅ Full project generationPhase 9: Advanced Features4-6 weeks✅ Diff/merge, hot reload, componentsPhase 10: Polish & Release2-4 weeks✅ Production ready
Total: ~6-9 months for full implementation

🎯 IMMEDIATE NEXT STEPS (This Week)
Day 1-2: Stateful Parser Foundation
bash# Create the stateful parser module
touch packages/forge_engine/src/stateful_parser.rs

# Add test fixture
mkdir -p packages/forge_engine/fixtures/stateful
# Create counter.dart fixture

# Write parser for state variables
Day 3-4: Graph Schema Update
bash# Update graph schema
vi forge_spec/graph_schema.json
# Add stateful_widget type

# Update renderer
vi packages/forge_engine/src/flutter_renderer.rs
# Add render_stateful_widget function
Day 5: Integration & Testing
bash# Wire everything together
# Run tests
cargo test stateful

# Test CLI
forge_cli import --file fixtures/stateful/counter.dart

💡 KEY SUCCESS METRICS
Phase 1 Success Criteria:

✅ Parse 10 common stateful patterns
✅ Round-trip preserves state variables
✅ Lifecycle methods intact
✅ setState() calls preserved

Phase 6 Success Criteria:

✅ Load and visualize any Flutter project
✅ Navigate between screens in graph
✅ Show state dependencies
✅ Interactive pan/zoom

Final Success Criteria:

✅ Import real-world Flutter app (1000+ LOC)
✅ Visualize full flow
✅ Make visual edits
✅ Export back to working Dart code
✅ Original app and exported app identical


🚀 What To Focus On NOW
Top Priority:
🚀 What To Focus On NOW (Continued)
Top Priority:

Complete StatefulWidget Support (Phase 1)

This is your foundation for everything else
Without this, you can't handle 80% of real Flutter apps
Start here, validate it works perfectly


Create Comprehensive Test Suite

Every feature needs fixtures that prove it works
Build regression tests as you go
Don't move forward until current phase passes all tests


Incremental Validation

After each phase, test on a real app
Start small (todo app) → medium (weather app) → large (e-commerce)
Find breaking patterns early




📦 RECOMMENDED PROJECT STRUCTURE (Final)
forge/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
├── docs/
│   ├── architecture/
│   │   ├── overview.md
│   │   ├── parser.md
│   │   ├── graph-format.md
│   │   └── renderer.md
│   ├── guides/
│   │   ├── getting-started.md
│   │   ├── stateful-widgets.md
│   │   ├── state-management.md
│   │   └── navigation.md
│   └── examples/
│       └── sample-apps/
│
├── fixtures/
│   ├── 01_basic_stateless/
│   │   ├── simple_text.dart
│   │   ├── nested_layout.dart
│   │   └── styled_widgets.dart
│   ├── 02_stateful/
│   │   ├── counter.dart
│   │   ├── form.dart
│   │   └── animation.dart
│   ├── 03_provider/
│   │   ├── simple_provider/
│   │   └── multi_provider/
│   ├── 04_navigation/
│   │   ├── basic_navigator/
│   │   └── named_routes/
│   ├── 05_real_world/
│   │   ├── todo_app/
│   │   ├── weather_app/
│   │   └── ecommerce_app/
│   └── 06_edge_cases/
│       ├── deeply_nested.dart
│       ├── dynamic_builder.dart
│       └── custom_render.dart
│
├── packages/
│   ├── forge_engine/          # Rust core
│   │   ├── benches/
│   │   ├── src/
│   │   │   ├── parser/
│   │   │   │   ├── dart_parser.rs
│   │   │   │   ├── stateless_parser.rs
│   │   │   │   ├── stateful_parser.rs
│   │   │   │   └── expression_parser.rs
│   │   │   ├── analyzer/
│   │   │   │   ├── project_analyzer.rs
│   │   │   │   ├── state_detector.rs
│   │   │   │   ├── navigation_analyzer.rs
│   │   │   │   └── conditional_analyzer.rs
│   │   │   ├── graph/
│   │   │   │   ├── graph_builder.rs
│   │   │   │   ├── graph_mutator.rs
│   │   │   │   └── graph_validator.rs
│   │   │   ├── renderer/
│   │   │   │   ├── flutter_renderer.rs
│   │   │   │   ├── stateless_renderer.rs
│   │   │   │   ├── stateful_renderer.rs
│   │   │   │   └── project_exporter.rs
│   │   │   ├── state/
│   │   │   │   ├── state_flow.rs
│   │   │   │   └── provider_graph.rs
│   │   │   ├── diff/
│   │   │   │   ├── diff_engine.rs
│   │   │   │   └── merge_engine.rs
│   │   │   ├── types.rs
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   ├── parser_tests.rs
│   │   │   ├── stateful_tests.rs
│   │   │   ├── project_tests.rs
│   │   │   └── integration_tests.rs
│   │   └── Cargo.toml
│   │
│   ├── forge_cli/             # Dart CLI
│   │   ├── bin/
│   │   │   └── forge_cli.dart
│   │   ├── lib/
│   │   │   ├── src/
│   │   │   │   ├── commands/
│   │   │   │   │   ├── import_command.dart
│   │   │   │   │   ├── export_command.dart
│   │   │   │   │   ├── analyze_command.dart
│   │   │   │   │   └── mutate_command.dart
│   │   │   │   ├── services/
│   │   │   │   │   └── forge_engine_bridge.dart
│   │   │   │   └── models/
│   │   │   │       ├── project_analysis.dart
│   │   │   │       └── graph_models.dart
│   │   │   └── forge_cli.dart
│   │   ├── test/
│   │   └── pubspec.yaml
│   │
│   └── forge_studio/          # Flutter UI
│       ├── lib/
│       │   ├── main.dart
│       │   ├── models/
│       │   │   ├── app_state.dart
│       │   │   ├── project_model.dart
│       │   │   ├── graph_model.dart
│       │   │   └── selection_model.dart
│       │   ├── screens/
│       │   │   ├── project_overview_screen.dart
│       │   │   ├── flow_graph_screen.dart
│       │   │   ├── screen_editor_screen.dart
│       │   │   └── state_graph_screen.dart
│       │   ├── widgets/
│       │   │   ├── canvas/
│       │   │   │   ├── flow_canvas.dart
│       │   │   │   ├── screen_node.dart
│       │   │   │   ├── navigation_edge.dart
│       │   │   │   └── state_flow_edge.dart
│       │   │   ├── panels/
│       │   │   │   ├── component_palette.dart
│       │   │   │   ├── property_inspector.dart
│       │   │   │   ├── widget_tree_panel.dart
│       │   │   │   └── code_preview.dart
│       │   │   └── shared/
│       │   │       ├── toolbar.dart
│       │   │       └── sidebar.dart
│       │   └── services/
│       │       ├── forge_engine_service.dart
│       │       ├── graph_editor_service.dart
│       │       ├── layout_service.dart
│       │       └── hot_reload_service.dart
│       ├── test/
│       └── pubspec.yaml
│
├── scripts/
│   ├── setup.sh
│   ├── build_engine.sh
│   ├── run_tests.sh
│   ├── run_integration_tests.sh
│   └── benchmark.sh
│
├── forge_spec/
│   ├── graph_schema.json
│   ├── logic_flow_v1.json
│   ├── analysis_report.schema.json
│   └── widget_registry.json
│
├── Cargo.toml              # Workspace
├── melos.yaml              # Dart workspace
├── README.md
└── ROADMAP.md

🧪 TESTING STRATEGY
Unit Tests (Per Module)
rust// packages/forge_engine/tests/stateful_tests.rs

#[cfg(test)]
mod stateful_parser_tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_counter() {
        let code = r#"
            class Counter extends StatefulWidget {
                @override
                State<Counter> createState() => _CounterState();
            }
            
            class _CounterState extends State<Counter> {
                int _count = 0;
                
                void _increment() {
                    setState(() {
                        _count++;
                    });
                }
                
                @override
                Widget build(BuildContext context) {
                    return Text('$_count');
                }
            }
        "#;
        
        let result = parse_stateful_widget(code).unwrap();
        
        assert_eq!(result.widget_class, "Counter");
        assert_eq!(result.state_class, "_CounterState");
        assert_eq!(result.state_variables.len(), 1);
        assert_eq!(result.state_variables[0].name, "_count");
        assert_eq!(result.mutations.len(), 1);
        assert_eq!(result.mutations[0].method_name, "_increment");
    }
    
    #[test]
    fn test_parse_with_lifecycle() {
        // Test initState and dispose
    }
    
    #[test]
    fn test_parse_animation_controller() {
        // Test complex stateful with AnimationController
    }
}
Integration Tests (Full Pipeline)
rust// packages/forge_engine/tests/integration_tests.rs

#[test]
fn test_full_round_trip_stateful() {
    let input_path = "fixtures/stateful/counter.dart";
    let temp_dir = tempdir().unwrap();
    let graph_path = temp_dir.path().join("graph.json");
    let output_path = temp_dir.path().join("output.dart");
    
    // Import
    let analysis = import_file(input_path).unwrap();
    save_graph(&analysis, &graph_path).unwrap();
    
    // Export
    let loaded = load_graph(&graph_path).unwrap();
    export_code(&loaded, &output_path).unwrap();
    
    // Compare
    let original = fs::read_to_string(input_path).unwrap();
    let generated = fs::read_to_string(output_path).unwrap();
    
    assert_code_equivalent(&original, &generated);
}

#[test]
fn test_full_project_analysis() {
    let project_path = "fixtures/real_world/todo_app";
    
    let analysis = analyze_project(project_path).unwrap();
    
    // Verify all screens found
    assert!(analysis.screens.len() >= 3);
    
    // Verify navigation detected
    assert!(!analysis.navigation.transitions.is_empty());
    
    // Verify providers found
    assert!(analysis.providers.len() >= 1);
}
End-to-End Tests (CLI)
bash#!/bin/bash
# scripts/run_integration_tests.sh

set -e

echo "🧪 Running E2E tests..."

# Test 1: Simple import/export
echo "Test 1: Basic round-trip"
./forge_cli import --file fixtures/stateful/counter.dart --output /tmp/graph.json
./forge_cli export --graph /tmp/graph.json --output /tmp/output.dart
diff fixtures/stateful/counter.dart /tmp/output.dart && echo "✅ Pass" || echo "❌ Fail"

# Test 2: Full project analysis
echo "Test 2: Project analysis"
./forge_cli analyze --project fixtures/real_world/todo_app
test -f forge_analysis.json && echo "✅ Pass" || echo "❌ Fail"

# Test 3: Code generation
echo "Test 3: Full export"
./forge_cli export --project fixtures/real_world/todo_app --output /tmp/exported_app
cd /tmp/exported_app
flutter pub get
flutter analyze && echo "✅ Pass" || echo "❌ Fail"
flutter test && echo "✅ Pass" || echo "❌ Fail"

echo "🎉 All tests passed!"
```

---

## 🎨 FORGE STUDIO UI MOCKUP

### Main Window Layout
```
┌─────────────────────────────────────────────────────────────────────┐
│  Forge Studio                                    [_] [□] [X]         │
├─────────────────────────────────────────────────────────────────────┤
│ File  Edit  View  Project  Tools  Help                              │
├───────────┬─────────────────────────────────────────┬───────────────┤
│           │                                         │               │
│  📁 Files │         🎨 FLOW CANVAS                  │ 🔧 Inspector  │
│           │                                         │               │
│  Screens  │    ┌─────────┐                         │  Selected:    │
│  ▼ lib/   │    │  Home   │                         │  HomeScreen   │
│    main   │    │ Screen  │──────┐                  │               │
│    ├─home │    └─────────┘      │                  │  Properties:  │
│    ├─login│                     │                  │  • title      │
│    └─prof │    ┌─────────┐      │                  │  • showAppBar │
│           │    │ Profile │◄─────┘                  │               │
│  Widgets  │    │ Screen  │                         │  State:       │
│  ▼ custom │    └─────────┘                         │  • _isLoading │
│    button │                                         │  • _userData  │
│           │         ┌──────────────┐               │               │
│  Providers│         │ AuthProvider │               │  Events:      │
│  ▼ state/ │         │  isLoggedIn  │───────┐       │  • onRefresh  │
│    auth   │         └──────────────┘       │       │               │
│    user   │                                 ↓       │  Code:        │
│           │    ┌─────────┐          ┌─────────┐    │  [View Code]  │
│  [+] New  │    │  Login  │          │Dashboard│    │               │
│           │    │ Screen  │◄─────────│ Screen  │    │               │
│           │    └─────────┘          └─────────┘    │               │
│           │                                         │               │
├───────────┴─────────────────────────────────────────┴───────────────┤
│  ⚡ Ready  │  📊 5 screens  │  🔗 8 routes  │  💾 Unsaved changes   │
└─────────────────────────────────────────────────────────────────────┘
```

### Flow Graph View (Detailed)
```
Navigation Flow:
┌──────────┐
│ App Start│
└─────┬────┘
      │
      ↓
   ┌──────────────┐
   │  Is Logged   │ ◄─── Conditional
   │     In?      │
   └──┬────────┬──┘
      │        │
   No │        │ Yes
      ↓        ↓
┌──────────┐ ┌──────────┐
│  Login   │ │Dashboard │
│  Screen  │ │  Screen  │
└─────┬────┘ └────┬─────┘
      │           │
      │ success   │ [Profile]
      │           ↓
      │      ┌──────────┐
      └─────>│ Profile  │
             │  Screen  │
             └──────────┘

State Flow:
┌────────────────┐
│ AuthProvider   │
│ • _isLoggedIn  │
│ • _user        │
└───────┬────────┘
        │
        │ notifyListeners()
        ↓
    ┌───────────────┐
    │  Consumer     │
    │  Widgets (3)  │
    └───┬───────────┘
        │
        ├─> Dashboard.UserAvatar
        ├─> Profile.UserInfo
        └─> Sidebar.LoginButton

🔥 CRITICAL IMPLEMENTATION PATTERNS
Pattern 1: Black Box Preservation
rust// When you can't fully parse something, preserve it

pub enum WidgetImplementation {
    FullyParsed {
        widget_tree: WidgetTree,
    },
    PartiallyParsed {
        known_parts: WidgetTree,
        unknown_parts: Vec<BlackBox>,
    },
    BlackBox {
        source_code: String,
        interface: WidgetInterface,  // What we know about it
    },
}

pub struct WidgetInterface {
    pub name: String,
    pub constructor_params: Vec<Param>,
    pub has_state: bool,
}

// Example usage
impl Parser {
    fn parse_widget(&self, code: &str) -> WidgetImplementation {
        if self.can_fully_parse(code) {
            FullyParsed { ... }
        } else if self.can_partially_parse(code) {
            PartiallyParsed { ... }
        } else {
            // Don't lose the code!
            BlackBox {
                source_code: code.to_string(),
                interface: extract_interface(code),
            }
        }
    }
}
Pattern 2: Incremental Analysis
rust// Don't re-parse unchanged files

pub struct ProjectCache {
    file_hashes: HashMap<PathBuf, String>,
    parsed_results: HashMap<PathBuf, ParseResult>,
}

impl ProjectCache {
    pub fn analyze_with_cache(&mut self, files: &[PathBuf]) -> Vec<ParseResult> {
        let mut results = Vec::new();
        
        for file in files {
            let content = fs::read_to_string(file)?;
            let hash = calculate_hash(&content);
            
            if let Some(cached_hash) = self.file_hashes.get(file) {
                if cached_hash == &hash {
                    // File unchanged, use cached result
                    results.push(self.parsed_results[file].clone());
                    continue;
                }
            }
            
            // Parse and cache
            let result = parse_file(&content)?;
            self.file_hashes.insert(file.clone(), hash);
            self.parsed_results.insert(file.clone(), result.clone());
            results.push(result);
        }
        
        results
    }
}
Pattern 3: Defensive Code Generation
rust// Always generate valid code, even if incomplete

impl CodeGenerator {
    pub fn render_widget(&self, widget: &Widget) -> String {
        let mut code = String::new();
        
        // Try to generate
        match self.try_render(widget) {
            Ok(generated) => code = generated,
            Err(e) => {
                // Fall back to preserved source
                code = widget.preserved_source.clone()
                    .unwrap_or_else(|| {
                        // Last resort: comment placeholder
                        format!(
                            "// TODO: Failed to generate {}\n// Error: {}\nContainer()",
                            widget.name, e
                        )
                    });
            }
        }
        
        code
    }
}
Pattern 4: Type-Safe Graph Mutations
rust// Make invalid states unrepresentable

pub struct GraphEditor {
    graph: ProjectGraph,
    undo_stack: Vec<GraphSnapshot>,
}

impl GraphEditor {
    // All mutations return Result and create undo points
    pub fn add_widget(
        &mut self,
        screen_id: ScreenId,
        parent_id: WidgetId,
        widget: Widget,
    ) -> Result<WidgetId> {
        // Validate
        self.validate_widget_addition(screen_id, parent_id, &widget)?;
        
        // Create undo point
        self.undo_stack.push(self.graph.snapshot());
        
        // Perform mutation
        let widget_id = self.graph.add_widget(screen_id, parent_id, widget)?;
        
        // Validate graph still consistent
        self.graph.validate()?;
        
        Ok(widget_id)
    }
    
    pub fn undo(&mut self) -> Result<()> {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.graph.restore(snapshot);
        }
        Ok(())
    }
}

📈 SUCCESS METRICS & KPIs
Development Metrics

Code Coverage: Target 80%+ for core parser
Test Pass Rate: 100% before each commit
Parse Success Rate: % of Flutter widgets successfully parsed
Round-Trip Fidelity: % of code identical after export

Performance Metrics

Parse Speed: < 100ms per 1000 LOC
Analysis Speed: < 5s for projects with 100 files
UI Responsiveness: 60 FPS in Studio canvas
Memory Usage: < 500MB for medium projects

Quality Metrics

Generated Code Validity: 100% pass dart analyze
Generated Code Compile: 100% pass flutter build
Test Preservation: 100% of tests still pass after round-trip

User Metrics (Post-Release)

Time Saved: Measure dev hours saved per project
Adoption Rate: % of Flutter devs using Forge
Satisfaction Score: NPS or CSAT surveys


🚨 COMMON PITFALLS TO AVOID
1. Over-Engineering Too Early
❌ Don't build AI features before basic parsing works
✅ Get the core loop working first: parse → edit → export
2. Ignoring Edge Cases
❌ Don't assume all Flutter code follows best practices
✅ Test against messy, real-world codebases
3. Perfect Parsing Syndrome
❌ Don't try to parse 100% of Dart AST perfectly
✅ Use black boxes for complex cases, focus on common patterns
4. Premature Optimization
❌ Don't optimize parser before it's correct
✅ Make it work, then make it fast
5. Scope Creep
❌ Don't add React/Vue/Angular support before Flutter works
✅ Master one framework completely first
6. Ignoring Developer Experience
❌ Don't build tools only designers can use
✅ Keep code ownership and Git workflow first-class
7. No Real-World Testing
❌ Don't only test on your own fixtures
✅ Test on popular open-source Flutter apps

🎯 MILESTONES & DEMO CHECKPOINTS
Milestone 1: "Hello World" (Week 4)
Demo: Import simple stateless widget, export identical code
bashforge_cli import --file hello.dart
forge_cli export --graph hello.json
diff hello.dart hello_output.dart  # No differences
Milestone 2: "Stateful Works" (Week 8)
Demo: Round-trip a counter app with setState
bashforge_cli import --file counter_app.dart
# Manually edit JSON to change initial count
forge_cli export --graph counter_modified.json
flutter run counter_output.dart  # Works perfectly
Milestone 3: "Real App" (Week 16)
Demo: Analyze todo app, show flow graph
bashforge_cli analyze --project ./todo_app
# Opens Studio showing:
# - 5 screens
# - Navigation flows
# - Provider connections
```

### Milestone 4: "Visual Editor" (Week 24)
**Demo:** Open app in Studio, drag-drop widget, regenerate
```
1. Load project in Forge Studio
2. Select HomeScreen
3. Drag FloatingActionButton to screen
4. Edit its color property
5. Click "Export Code"
6. Run exported app - new button appears with correct color
```

### Milestone 5: "Production Ready" (Week 36)
**Demo:** Import popular OSS app (e.g., FlutterFire UI), visualize, export
```
1. Clone firebase/flutterfire
2. forge_cli analyze --project ./packages/firebase_ui_auth
3. Studio shows complete auth flow
4. Export with modifications
5. All tests still pass

📞 WHEN TO ASK FOR HELP
Get External Review At:

After Phase 1: AST parsing strategy
After Phase 3: State management detection approach
After Phase 6: Studio UX/UI design
After Phase 8: Code generation quality

Find Beta Testers At:

Flutter community forums
Reddit /r/FlutterDev
Discord Flutter servers
Local Flutter meetups

Open Source Strategy:

Open source the core engine (Phase 1-5)
Build community around parsers
Premium features in Studio (Phase 6+)
Marketplace takes commission (future)


✅ FINAL CHECKLIST (Before Each Phase)
Before Starting Phase:

 Previous phase tests all pass
 Documentation updated
 Fixtures created for new features
 Architecture design reviewed
 Breaking changes documented

Before Completing Phase:

 All new tests passing
 Integration tests added
 Performance benchmarks run
 Example updated in README
 Git tagged with version

Before Public Release:

 Security audit completed
 License chosen (MIT/Apache)
 Contributing guide written
 Code of conduct added
 CI/CD pipeline working
 Release notes written
 Marketing materials ready


🎉 YOU'RE READY TO START!
Your immediate action plan:
bash# Week 1: Setup
cd packages/forge_engine
touch src/stateful_parser.rs
mkdir -p fixtures/stateful
# Create counter.dart fixture

# Week 2-3: Implementation
# Write stateful parser
# Write tests
# Update renderer

# Week 4: Validation
cargo test
./forge_cli import --file fixtures/stateful/counter.dart
# Demo working round-trip

# Week 5: Move to Phase 2
# Start project-level analysis
Remember:

Build incrementally
Test continuously
Don't skip phases
Ask for feedback early
Ship small, ship often