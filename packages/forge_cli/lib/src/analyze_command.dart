import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';

import 'engine_bridge.dart';
import 'workspace_context.dart';

class AnalyzeCommand extends Command<int> {
  AnalyzeCommand(this._workspace) {
    argParser
      ..addOption(
        'file',
        help: 'Path to the Dart file to analyze.',
      )
      ..addOption(
        'output',
        help:
            'Optional file path to write the merged analysis report JSON. Defaults to stdout.',
      )
      ..addOption(
        'confidence',
        help:
            'Native confidence threshold (0.0-1.0). Below this, the analyzer fallback runs.',
      );
  }

  final WorkspaceContext _workspace;

  @override
  String get name => 'analyze';

  @override
  String get description =>
      'Run the hybrid analyzer + merge pipeline using the Rust engine.';

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

    final args = <String>['analyze', '--file', input.absolute.path];

    final confidenceOption = argResults?['confidence'] as String?;
    if (confidenceOption != null) {
      final parsed = double.tryParse(confidenceOption);
      if (parsed == null || parsed.isNaN || parsed < 0 || parsed > 1) {
        throw UsageException(
            '--confidence must be a number between 0.0 and 1.0', usage);
      }
      args
        ..add('--confidence')
        ..add(parsed.toString());
    }

    final outputOption = argResults?['output'] as String?;
    File? outputFile;
    if (outputOption != null) {
      outputFile = File(outputOption);
      args
        ..add('--output')
        ..add(outputFile.absolute.path);
    }

    final engineBinary = resolveEngineBinary(_workspace);
    final processResult = await Process.run(
      engineBinary,
      args,
      stdoutEncoding: utf8,
      stderrEncoding: utf8,
    );

    if (processResult.stdout is String &&
        (processResult.stdout as String).isNotEmpty) {
      stdout.write(processResult.stdout);
    }
    if (processResult.stderr is String &&
        (processResult.stderr as String).isNotEmpty) {
      stderr.write(processResult.stderr);
    }

    // Pick up report from file or stdout for summary output.
    final reportJson =
        await _loadReport(outputFile, processResult.stdout as String?);
    if (reportJson != null) {
      _printSummary(reportJson);
    }

    return processResult.exitCode;
  }

  Future<Map<String, dynamic>?> _loadReport(
      File? outputFile, String? stdoutJson) async {
    try {
      if (outputFile != null) {
        if (!await outputFile.exists()) {
          stderr.writeln(
              'Warning: analysis output file ${outputFile.path} was not created.');
          return null;
        }
        final contents = await outputFile.readAsString();
        return jsonDecode(contents) as Map<String, dynamic>;
      }

      if (stdoutJson == null || stdoutJson.trim().isEmpty) {
        return null;
      }

      return jsonDecode(stdoutJson) as Map<String, dynamic>;
    } on FormatException catch (e) {
      stderr.writeln('Failed to parse analysis report JSON: $e');
      return null;
    }
  }

  void _printSummary(Map<String, dynamic> report) {
    final totalConflicts = report['total_conflicts'] as int? ?? 0;
    final outcomes = report['outcomes'] as List<dynamic>? ?? const [];

    if (totalConflicts == 0) {
      stdout.writeln('✅ Merge completed with no conflicts.');
      return;
    }

    stdout.writeln('⚠️  Merge completed with $totalConflicts conflict(s).');
    for (final outcome in outcomes) {
      final merge = outcome is Map<String, dynamic>
          ? outcome['merge'] as Map<String, dynamic>?
          : null;
      final conflicts = merge?['conflicts'] as List<dynamic>? ?? const [];
      final quick = outcome is Map<String, dynamic>
          ? outcome['quick_graph'] as Map<String, dynamic>?
          : null;
      final screenId = quick?['id'] ?? '<unknown screen>';

      if (conflicts.isEmpty) {
        continue;
      }

      stdout.writeln('  Screen: $screenId');
      for (final conflict in conflicts) {
        final conflictMap = conflict as Map<String, dynamic>;
        final path = conflictMap['path'] ?? '<unknown path>';
        stdout.writeln('    • $path');
      }
    }
  }
}
