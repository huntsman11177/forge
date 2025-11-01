import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';
import 'package:json_schema/json_schema.dart';
import 'package:path/path.dart' as p;

import 'workspace_context.dart';
import 'importers/flutter_parser.dart';

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
        help:
            'Optional file path to write ForgeGraph JSON. Defaults to stdout.',
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

    final parser = const FlutterParser();
    final document = await parser.parseScreen(input.absolute.path);

    final schemaPath = p.join(
      _workspace.workspaceRoot,
      'forge_spec',
      'graph_schema.json',
    );

    ValidationResults results;
    try {
      results = await FlutterParser.validateGraphSchema(
        document,
        schemaPath: schemaPath,
      );
    } on FlutterParserSchemaError catch (error) {
      stderr.writeln('Schema validation setup failed: ${error.message}');
      return 74; // EX_IOERR
    }

    if (!results.isValid) {
      stderr.writeln('❌ Schema validation failed:');
      for (final issue in results.errors) {
        final pointer = issue.instancePath.toString();
        final path = pointer.isEmpty ? '/' : pointer;
        stderr.writeln('  - $path: ${issue.message}');
      }
      return 2;
    }

    final payload = const JsonEncoder.withIndent('  ').convert(document);

    final outputPath = argResults?['output'] as String?;
    if (outputPath != null) {
      final outFile = File(outputPath);
      await outFile.create(recursive: true);
      await outFile.writeAsString(payload);
      stdout.writeln('✅ Schema validation passed.');
    } else {
      stderr.writeln('✅ Schema validation passed.');
      stdout.writeln(payload);
    }

    return 0;
  }
}

