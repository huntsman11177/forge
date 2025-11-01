import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';
import 'package:path/path.dart' as p;

import 'graph_diff.dart';
import 'importers/flutter_parser.dart';
import 'workspace_context.dart';

class DiffCommand extends Command<int> {
  DiffCommand(this._workspace) {
    argParser
      ..addOption(
        'baseline',
        abbr: 'b',
        help: 'Path to the baseline Forge graph JSON file.',
        valueHelp: 'graph.json',
      )
      ..addOption(
        'file',
        abbr: 'f',
        help: 'Path to the Dart file to import and diff against the baseline.',
        valueHelp: 'screen.dart',
      )
      ..addOption(
        'output',
        abbr: 'o',
        help: 'Optional file path to write structured diff results as JSON.',
        valueHelp: 'diff.json',
      );
  }

  final WorkspaceContext _workspace;

  @override
  String get name => 'diff';

  @override
  String get description =>
      'Compare an imported Forge graph against a baseline JSON graph.';

  @override
  Future<int> run() async {
    final baselinePath = argResults?['baseline'] as String?;
    final filePath = argResults?['file'] as String?;

    if (baselinePath == null || filePath == null) {
      throw UsageException('Both --baseline and --file are required.', usage);
    }

    final baselineFile = File(baselinePath);
    if (!await baselineFile.exists()) {
      stderr.writeln('Baseline graph not found: ${baselineFile.path}');
      return 66; // EX_NOINPUT
    }

    Map<String, dynamic> baselineGraph;
    try {
      final raw = await baselineFile.readAsString();
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        stderr.writeln('Baseline graph must be a JSON object.');
        return 65; // EX_DATAERR
      }
      baselineGraph = decoded;
    } on FormatException catch (error) {
      stderr.writeln('Failed to decode baseline JSON: ${error.message}');
      return 65; // EX_DATAERR
    }

    final inputFile = File(filePath);
    if (!await inputFile.exists()) {
      stderr.writeln('Input Dart file not found: ${inputFile.path}');
      return 66; // EX_NOINPUT
    }

    final parser = const FlutterParser();
    Map<String, dynamic> candidateGraph;
    try {
      candidateGraph = await parser.parseScreen(inputFile.absolute.path);
    } catch (error) {
      stderr.writeln('Failed to parse candidate file: $error');
      return 70; // EX_SOFTWARE
    }

    final schemaPath = p.join(
      _workspace.workspaceRoot,
      'forge_spec',
      'graph_schema.json',
    );

    try {
      final validation = await FlutterParser.validateGraphSchema(
        candidateGraph,
        schemaPath: schemaPath,
      );
      if (!validation.isValid) {
        stderr.writeln('❌ Candidate graph failed schema validation:');
        for (final issue in validation.errors) {
          final pointer = issue.instancePath.toString();
          final path = pointer.isEmpty ? '/' : pointer;
          stderr.writeln('  - $path: ${issue.message}');
        }
        return 2;
      }
    } on FlutterParserSchemaError catch (error) {
      stderr.writeln('Schema validation setup failed: ${error.message}');
      return 74; // EX_IOERR
    }

    final diff = GraphDiffer.diff(baselineGraph, candidateGraph);
    final diffSummary = {
      'changes': diff.changes,
      'details': diff.toJson(),
    };

    final outputPath = argResults?['output'] as String?;
    if (outputPath != null && outputPath.isNotEmpty) {
      final outFile = File(outputPath);
      await outFile.create(recursive: true);
      await outFile.writeAsString(
        const JsonEncoder.withIndent('  ').convert(diffSummary),
      );
      stdout.writeln('📝 Diff results written to ${outFile.path}');
    }

    if (!diff.hasChanges) {
      stdout.writeln('✅ No graph differences detected.');
      return 0;
    }

    stdout.writeln('⚠️ Graph differences detected:');
    for (final change in diff.changes) {
      stdout.writeln('  - $change');
    }

    return 3; // Differences found.
  }
}
