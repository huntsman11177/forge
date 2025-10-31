import 'dart:io';

import 'package:args/command_runner.dart';

import 'engine_bridge.dart';
import 'workspace_context.dart';

class ExportCommand extends Command<int> {
  ExportCommand(this._workspace) {
    argParser
      ..addOption(
        'file',
        abbr: 'f',
        help: 'Path to the Forge graph JSON file to export.',
        mandatory: true,
      )
      ..addOption(
        'out',
        abbr: 'o',
        help: 'Optional output path. Defaults to stdout when omitted.',
      )
      ..addOption(
        'project-id',
        help: 'Override the project identifier embedded in the export.',
      )
      ..addOption(
        'project-name',
        help: 'Override the project name embedded in the export.',
      );
  }

  final WorkspaceContext _workspace;

  @override
  String get name => 'export';

  @override
  String get description =>
      'Emit canonical Forge schema JSON from a graph file using the Rust engine.';

  @override
  Future<int> run() async {
    final filePath = argResults!['file'] as String;
    final input = File(filePath);
    if (!await input.exists()) {
      stderr.writeln('Input graph not found: ${input.path}');
      return 66; // EX_NOINPUT
    }

    final args = <String>['export', '--file', input.absolute.path];

    final projectId = argResults?['project-id'] as String?;
    if (projectId != null && projectId.isNotEmpty) {
      args
        ..add('--project-id')
        ..add(projectId);
    }

    final projectName = argResults?['project-name'] as String?;
    if (projectName != null && projectName.isNotEmpty) {
      args
        ..add('--project-name')
        ..add(projectName);
    }

    final outPath = argResults?['out'] as String?;
    if (outPath != null && outPath.isNotEmpty) {
      args
        ..add('--out')
        ..add(outPath);
    }

    final result = await runEngine(_workspace, args);

    if (result.exitCode != 0) {
      stderr
        ..writeln(result.stderr)
        ..writeln('Engine exited with code ${result.exitCode}');
      return result.exitCode;
    }

    if (outPath == null || outPath.isEmpty) {
      stdout.write(result.stdout);
    }

    return 0;
  }
}
