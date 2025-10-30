import 'package:flutter/material.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const ForgeStudioApp());
}

/// ForgeStudioApp bootstraps the Flutter Web runtime that will host the
/// executable canvas and developer tooling.
class ForgeStudioApp extends StatelessWidget {
  const ForgeStudioApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Forge Studio',
      theme: ThemeData.from(colorScheme: const ColorScheme.light()),
      darkTheme: ThemeData.from(colorScheme: const ColorScheme.dark()),
      themeMode: ThemeMode.system,
      home: const _BootstrapScreen(),
    );
  }
}

/// Temporary shell showcasing the live runtime placeholder. This widget is
/// intentionally functional and will be replaced by the canvas runtime once the
/// graph engine is integrated.
class _BootstrapScreen extends StatefulWidget {
  const _BootstrapScreen();

  @override
  State<_BootstrapScreen> createState() => _BootstrapScreenState();
}

class _BootstrapScreenState extends State<_BootstrapScreen> {
  String _statusMessage =
      'Forge Studio is loading the workspace. Connect a project to begin.';

  @override
  void initState() {
    super.initState();
    _initialize();
  }

  Future<void> _initialize() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    if (!mounted) return;
    setState(() {
      _statusMessage = 'Workspace ready. Import a Flutter project to continue.';
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.surface,
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 480),
          child: Card(
            elevation: 3,
            child: Padding(
              padding: const EdgeInsets.all(32),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Forge Studio',
                    style: Theme.of(context).textTheme.headlineMedium,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    'The live Flutter runtime will appear here as soon as the '
                    'Forge Engine provides a compiled UI graph.',
                    style: Theme.of(context).textTheme.bodyLarge,
                  ),
                  const SizedBox(height: 24),
                  Row(
                    children: [
                      const Icon(Icons.sync, size: 18),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          _statusMessage,
                          style: Theme.of(context).textTheme.bodyMedium,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 24),
                  FilledButton(
                    onPressed: () {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text('Import CLI integration coming soon.'),
                        ),
                      );
                    },
                    child: const Text('Import project'),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
