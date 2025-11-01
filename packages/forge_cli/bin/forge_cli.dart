import 'dart:io';

import 'package:args/command_runner.dart';
import 'package:forge_cli/src/analyze_command.dart';
import 'package:forge_cli/src/diff_command.dart';
import 'package:forge_cli/src/export_command.dart';
import 'package:forge_cli/src/import_command.dart';
import 'package:forge_cli/src/render_command.dart';
import 'package:forge_cli/src/version_command.dart';
import 'package:forge_cli/src/simulate_command.dart';
import 'package:forge_cli/src/workspace_context.dart';

Future<void> main(List<String> arguments) async {
  final workspace = await WorkspaceContext.find();
  final runner = CommandRunner<int>(
    'forge',
    'Forge tooling for import/export, validation, and packaging workflows.',
  )
    ..addCommand(ImportCommand(workspace))
    ..addCommand(DiffCommand(workspace))
    ..addCommand(ExportCommand(workspace))
    ..addCommand(RenderCommand(workspace))
    ..addCommand(AnalyzeCommand(workspace))
    ..addCommand(SimulateCommand(workspace))
    ..addCommand(VersionCommand(workspace));

  try {
    final result = await runner.run(arguments);
    exit(result ?? 0);
  } on UsageException catch (e) {
    stderr.writeln(e.message);
    stderr.writeln();
    stderr.writeln(e.usage);
    exitCode = 64; // EX_USAGE.
  } on WorkspaceContextException catch (e) {
    stderr.writeln('Workspace error: ${e.message}');
    exitCode = 74; // EX_IOERR.
  } catch (e, stackTrace) {
    stderr
      ..writeln('Unexpected error: $e')
      ..writeln(stackTrace);
    exitCode = 1;
  }
}
