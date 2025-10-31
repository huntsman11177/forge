import 'dart:convert';
import 'dart:io';

import 'package:args/command_runner.dart';

import 'engine_bridge.dart';
import 'workspace_context.dart';

class SimulateCommand extends Command<int> {
  SimulateCommand(this._workspace) {
    argParser
      ..addOption(
        'flow',
        help: 'Flow identifier to simulate (e.g. flow.basic).',
      )
      ..addOption(
        'graph',
        help: 'Path to logic graph JSON file (logic_flow_v1).',
      )
      ..addOption(
        'entry',
        help: 'Optional explicit entry node identifier.',
      )
      ..addOption(
        'providers',
        help: 'Optional JSON file providing seed provider state.',
      )
      ..addOption(
        'output',
        help: 'Optional file path to write the simulation result JSON.',
      )
      ..addOption(
        'max-steps',
        help: 'Override maximum scheduler steps (defaults to engine default).',
      )
      ..addOption(
        'max-trace',
        help: 'Override maximum trace entries retained (defaults to engine default).',
      );
  }

  final WorkspaceContext _workspace;

  @override
  String get name => 'simulate';

  @override
  String get description =>
      'Run a logic flow using the Forge engine and emit its evaluation trace.';

  @override
  Future<int> run() async {
    final flowId = argResults?['flow'] as String?;
    final graphPath = argResults?['graph'] as String?;
    if (flowId == null || graphPath == null) {
      throw UsageException('--flow and --graph are required', usage);
    }

    final graphFile = File(graphPath);
    if (!await graphFile.exists()) {
      stderr.writeln('Logic graph not found: ${graphFile.path}');
      return 66; // EX_NOINPUT
    }

    File? providerFile;
    final providersPath = argResults?['providers'] as String?;
    if (providersPath != null) {
      providerFile = File(providersPath);
      if (!await providerFile.exists()) {
        stderr.writeln('Providers file not found: ${providerFile.path}');
        return 66;
      }
    }

    final engineBinary = resolveEngineBinary(_workspace);
    final args = <String>[
      'simulate',
      '--flow',
      flowId,
      '--graph',
      graphFile.absolute.path,
    ];

    final entry = argResults?['entry'] as String?;
    if (entry != null) {
      args..add('--entry')..add(entry);
    }

    if (providerFile != null) {
      args..add('--providers')..add(providerFile.absolute.path);
    }

    final output = argResults?['output'] as String?;
    File? outputFile;
    if (output != null) {
      outputFile = File(output);
      args..add('--output')..add(outputFile.absolute.path);
    }

    final maxSteps = argResults?['max-steps'] as String?;
    if (maxSteps != null) {
      final parsed = int.tryParse(maxSteps);
      if (parsed == null || parsed <= 0) {
        throw UsageException('--max-steps must be a positive integer', usage);
      }
      args..add('--max-steps')..add(parsed.toString());
    }

    final maxTrace = argResults?['max-trace'] as String?;
    if (maxTrace != null) {
      final parsed = int.tryParse(maxTrace);
      if (parsed == null || parsed <= 0) {
        throw UsageException('--max-trace must be a positive integer', usage);
      }
      args..add('--max-trace')..add(parsed.toString());
    }

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

    if (processResult.exitCode != 0) {
      return processResult.exitCode;
    }

    Map<String, dynamic> payload;
    try {
      if (outputFile != null) {
        if (!await outputFile.exists()) {
          stderr.writeln(
              'Warning: simulation output file ${outputFile.path} was not created.');
          return processResult.exitCode;
        }
        final contents = await outputFile.readAsString();
        payload = jsonDecode(contents) as Map<String, dynamic>;
      } else {
        final stdoutJson = processResult.stdout as String?;
        if (stdoutJson == null || stdoutJson.trim().isEmpty) {
          stderr.writeln(
              'Simulation succeeded but no JSON payload was emitted by the engine.');
          return processResult.exitCode;
        }
        payload = jsonDecode(stdoutJson) as Map<String, dynamic>;
      }
    } on FormatException catch (error) {
      stderr.writeln('Failed to parse simulation JSON: $error');
      return processResult.exitCode;
    }

    _printSummary(payload);
    return processResult.exitCode;
  }

  void _printSummary(Map<String, dynamic> payload) {
    final flowId = payload['flow_id'] ?? '<unknown flow>';
    final entry = payload['entry'] ?? '<implicit entry>';
    final result = payload['result'] as Map<String, dynamic>?;
    final success = result?['success'] == true;
    final diagnostics = result?['diagnostics'] as List<dynamic>? ?? const [];
    final trace = result?['traces'] as List<dynamic>? ?? const [];

    stdout.writeln(
        'Simulation ${success ? '✅ succeeded' : '⚠️  completed with errors'} for flow $flowId (entry: $entry).');
    stdout.writeln('  Trace entries: ${trace.length}');
    if (diagnostics.isNotEmpty) {
      stdout.writeln('  Diagnostics:');
      for (final diag in diagnostics) {
        stdout.writeln('    • $diag');
      }
    }

    final returnValue = result?['return_value'];
    if (returnValue != null) {
      stdout.writeln('  Return value: $returnValue');
    }
  }
}
