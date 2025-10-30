import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';
import 'package:path/path.dart' as p;

import 'workspace_context.dart';

/// Runs the Forge engine to parse Dart source into ForgeGraph JSON.
class ImportCommand extends Command<int> {
  ImportCommand(this._workspace) {
    argParser
      ..addOption(
        'file',
        help: 'Path to the Dart file to import.',
      )
      ..addOption(
        'output',
        help: 'Optional file path to write ForgeGraph JSON. Defaults to stdout.',
      );
  }

  final WorkspaceContext _workspace;

  @override
  String get name => 'import';

  @override
  String get description =>
      'Parse Flutter source into Forge graphs using the Rust engine.';

  @override
  Future<int> run() async {
    final filePath = argResults?['file'] as String?;
    if (filePath == null) {
      throw UsageException('--file is required', usage);
    }

    final input = File(filePath);
    if (!await input.exists()) {
      stderr.writeln('Input file not found: ${input.path}');
      return 66; // EX_NOINPUT
    }

    final engineBinary = _resolveEngineBinary();
    final processResult = await Process.run(
      engineBinary,
      ['--file', input.absolute.path],
      stdoutEncoding: utf8,
      stderrEncoding: utf8,
    );

    if (processResult.exitCode != 0) {
      stderr
        ..writeln(processResult.stderr)
        ..writeln('Engine exited with code ${processResult.exitCode}');
      return processResult.exitCode;
    }

    final outputPath = argResults?['output'] as String?;
    if (outputPath != null) {
      final outFile = File(outputPath);
      await outFile.create(recursive: true);
      await outFile.writeAsString(processResult.stdout as String);
    } else {
      stdout.write(processResult.stdout);
    }

    return 0;
  }

  String _resolveEngineBinary() {
    final platform = Platform.operatingSystem;
    final binaryName = platform == 'windows'
        ? 'forge_engine_cli.exe'
        : 'forge_engine_cli';
    final binaryPath = p.join(
      _workspace.workspaceRoot,
      'packages',
      'forge_engine',
      'target',
      'release',
      binaryName,
    );

    if (!File(binaryPath).existsSync()) {
      throw WorkspaceContextException(
        'Forge engine binary not found at $binaryPath. Run `cargo build --release` '
        'inside packages/forge_engine to compile it.',
      );
    }
    return binaryPath;
  }
}
