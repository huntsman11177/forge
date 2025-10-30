import 'dart:io';

import 'package:path/path.dart' as p;

import 'workspace_context.dart';

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
