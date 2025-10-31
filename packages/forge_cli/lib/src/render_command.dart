import 'dart:io';

import 'package:args/command_runner.dart';

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

    final args = <String>[
      'render',
      '--file',
      input.absolute.path,
      '--framework',
      framework,
    ];

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
  }
}
