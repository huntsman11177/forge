import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';
import 'package:path/path.dart' as p;

import 'engine_bridge.dart';
import 'workspace_context.dart';

const _supportedFrameworks = ['flutter', 'react', 'angular'];
const _defaultFramework = 'flutter';

class RenderCommand extends Command<int> {
  RenderCommand(this._workspace) {
    argParser
      ..addOption(
        'file',
        abbr: 'f',
        help: 'Path to the Forge UI graph JSON file.',
        mandatory: true,
      )
      ..addOption(
        'framework',
        abbr: 't',
        allowed: _supportedFrameworks,
        defaultsTo: _defaultFramework,
        help:
            'Target framework to render (${_supportedFrameworks.join(', ')}).',
      )
      ..addOption(
        'out-dir',
        abbr: 'o',
        help: 'Optional output directory for rendered code.',
      )
      ..addFlag(
        'emit-manifest',
        help:
            'Emit framework-specific dependency manifest (e.g., pubspec.yaml, package.json).',
        defaultsTo: false,
      );
  }

  final WorkspaceContext _workspace;

  @override
  String get name => 'render';

  @override
  String get description =>
      'Render a Forge graph to target framework source code.';

  @override
  Future<int> run() async {
    final fileOption = argResults?['file'] as String?;
    if (fileOption == null) {
      throw UsageException('--file is required', usage);
    }

    final input = File(fileOption);
    if (!await input.exists()) {
      stderr.writeln('Graph file not found: ${input.path}');
      return 66; // EX_NOINPUT
    }

    final framework =
        (argResults?['framework'] as String?) ?? _defaultFramework;
    final outDir = argResults?['out-dir'] as String?;

    final args = <String>['render'];
    Directory? tempDir;
    File? tempFile;

    try {
      final decoded = await _tryDecodeCanonicalDocument(input);
      if (decoded != null) {
        final screens = decoded['screens'] as List<dynamic>?;
        if (screens == null || screens.isEmpty) {
          stderr.writeln('Canonical document does not contain any screens.');
          return 65; // EX_DATAERR
        }

        tempDir = await Directory.systemTemp.createTemp('forge_render_');
        final screen = screens.first as Map<String, dynamic>;
        final screenId = screen['id'] as String? ?? 'screen_0';
        final root = screen['root'] as Map<String, dynamic>?;
        if (root == null) {
          stderr.writeln('Screen $screenId is missing a root widget.');
          return 65; // EX_DATAERR
        }

        final screenGraph = <String, dynamic>{
          'id': screenId,
          'root': _normaliseWidgetNode(root),
        };

        tempFile = File(p.join(tempDir.path, '$screenId.json'));
        await tempFile.writeAsString(jsonEncode(screenGraph));

        args
          ..add('--file')
          ..add(tempFile.path);
      } else {
        args
          ..add('--file')
          ..add(input.absolute.path);
      }

      args
        ..add('--framework')
        ..add(framework);

      if (outDir != null && outDir.isNotEmpty) {
        args
          ..add('--out-dir')
          ..add(outDir);
      }

      if (argResults?.flag('emit-manifest') == true) {
        args.add('--emit-manifest');
      }

      final result = await runEngine(_workspace, args);

      final stdoutData = result.stdout;
      final stderrData = result.stderr;
      if (stdoutData is String && stdoutData.isNotEmpty) {
        stdout.write(stdoutData);
      }
      if (stderrData is String && stderrData.isNotEmpty) {
        stderr.write(stderrData);
      }

      return result.exitCode;
    } finally {
      if (tempFile != null && tempFile.existsSync()) {
        tempFile.deleteSync();
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

  return {
    'type': 'literal',
    'value': value,
  };
}
