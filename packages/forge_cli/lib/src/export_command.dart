import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';
import 'package:path/path.dart' as p;

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

    final args = <String>['export'];
    Directory? tempDir;
    final tempFiles = <File>[];

    try {
      final decoded = await _tryDecodeCanonicalDocument(input);
      if (decoded != null) {
        final screens = decoded['screens'] as List<dynamic>?;
        if (screens == null || screens.isEmpty) {
          stderr.writeln('Canonical document does not contain any screens.');
          return 65; // EX_DATAERR
        }

        tempDir = await Directory.systemTemp.createTemp('forge_export_');
        for (var index = 0; index < screens.length; index++) {
          final screen = screens[index] as Map<String, dynamic>;
          final screenId = screen['id'] as String? ?? 'screen_$index';
          final root = screen['root'] as Map<String, dynamic>?;
          if (root == null) {
            stderr.writeln('Screen $screenId is missing a root widget.');
            return 65; // EX_DATAERR
          }

          final screenGraph = <String, dynamic>{
            'id': screenId,
            'root': _normaliseWidgetNode(root),
          };

          final tempFile = File(p.join(tempDir.path, '$screenId.json'));
          await tempFile.writeAsString(jsonEncode(screenGraph));
          tempFiles.add(tempFile);

          args
            ..add('--file')
            ..add(tempFile.path);
        }

        final project = decoded['project'];
        if (project is Map<String, dynamic>) {
          final projectId = (project['id'] as String?)?.trim();
          if (projectId != null && projectId.isNotEmpty) {
            args
              ..add('--project-id')
              ..add(projectId);
          }
          final projectName = (project['name'] as String?)?.trim();
          if (projectName != null && projectName.isNotEmpty) {
            args
              ..add('--project-name')
              ..add(projectName);
          }
        }
      } else {
        args
          ..add('--file')
          ..add(input.absolute.path);
      }

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
    } finally {
      for (final file in tempFiles) {
        if (file.existsSync()) {
          file.deleteSync();
        }
      }
      if (tempDir != null && tempDir.existsSync()) {
        tempDir.deleteSync(recursive: true);
      }
    }
  }
}

Future<Map<String, dynamic>?> _tryDecodeCanonicalDocument(File input) async {
  try {
    final contents = await input.readAsString();
    final decoded = jsonDecode(contents);
    if (decoded is Map<String, dynamic> && decoded.containsKey('screens')) {
      return decoded;
    }
  } catch (_) {
    // ignore and treat as non-canonical
  }
  return null;
}

Map<String, dynamic> _normaliseWidgetNode(Map<String, dynamic> node) {
  final props = node['props'];
  final children = node['children'];

  final normalised = <String, dynamic>{
    'widget': node['widget'],
    'props': <String, dynamic>{},
    'children': <Map<String, dynamic>>[],
  };

  if (props is Map<String, dynamic>) {
    props.forEach((key, value) {
      final normalisedProp = _normalisePropValue(value);
      if (normalisedProp != null) {
        normalised['props'][key] = normalisedProp;
      }
    });
  }

  if (children is List) {
    for (final child in children) {
      if (child is Map<String, dynamic>) {
        normalised['children'].add(_normaliseWidgetNode(child));
      }
    }
  }

  return normalised;
}

Map<String, dynamic>? _normalisePropValue(dynamic value) {
  if (value is Map<String, dynamic>) {
    final type = value['type'];
    switch (type) {
      case 'literal':
        return {
          'type': 'literal',
          'value': value['value'],
        };
      case 'expression':
        return {
          'type': 'expression',
          'expression': value['expression'],
        };
      case 'binding':
        return {
          'type': 'binding',
          'binding': value['binding'],
        };
      case 'enum':
        final enumValue = value['value'];
        if (enumValue is String) {
          return {
            'type': 'expression',
            'expression': enumValue,
          };
        }
        break;
      default:
        break;
    }
  }

  // Fallback to literal representation
  return {
    'type': 'literal',
    'value': value,
  };
}
