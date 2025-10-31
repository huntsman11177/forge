import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import 'workspace_context.dart';

/// Executes the Forge engine binary with [arguments], returning
/// [ProcessResult]. The command runs to completion before returning.
Future<ProcessResult> runEngine(
  WorkspaceContext workspace,
  List<String> arguments,
) async {
  final binary = resolveEngineBinary(workspace);
  return Process.run(
    binary,
    arguments,
    stdoutEncoding: utf8,
    stderrEncoding: utf8,
  );
}

/// Attempts to read JSON either from [path] (if provided) or fallback [stdout].
/// Returns `null` when neither source yields a payload.
Future<Map<String, dynamic>?> readJsonPayload({
  File? path,
  String? stdout,
}) async {
  try {
    if (path != null) {
      if (!await path.exists()) {
        return null;
      }
      final contents = await path.readAsString();
      if (contents.trim().isEmpty) {
        return null;
      }
      return jsonDecode(contents) as Map<String, dynamic>;
    }

    if (stdout == null || stdout.trim().isEmpty) {
      return null;
    }

    return jsonDecode(stdout) as Map<String, dynamic>;
  } on FormatException {
    return null;
  }
}

/// Resolves the absolute path to the compiled Forge engine CLI binary.
String resolveEngineBinary(WorkspaceContext workspace) {
  final platform = Platform.operatingSystem;
  final binaryName =
      platform == 'windows' ? 'forge_engine_cli.exe' : 'forge_engine_cli';

  final binaryPath = p.join(
    workspace.workspaceRoot,
    'packages',
    'forge_engine',
    'target',
    'release',
    binaryName,
  );

  if (!File(binaryPath).existsSync()) {
    throw WorkspaceContextException(
      'Forge engine binary not found at $binaryPath. Run '
      '`cargo build --release` inside packages/forge_engine to compile it.',
    );
  }

  return binaryPath;
}
