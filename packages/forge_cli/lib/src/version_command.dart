import 'dart:async';

import 'package:args/command_runner.dart';

import 'workspace_context.dart';

/// Prints information about the current Forge workspace and tooling version.
class VersionCommand extends Command<int> {
  VersionCommand(this._workspace);

  final WorkspaceContext _workspace;

  @override
  String get name => 'version';

  @override
  String get description => 'Show the current Forge workspace version.';

  @override
  Future<int> run() async {
    if (argResults?.rest.isNotEmpty == true) {
      throw UsageException('version does not accept arguments', usage);
    }

    final buffer = StringBuffer()
      ..writeln('Forge workspace: ${_workspace.workspaceRoot}')
      ..writeln('Version: ${_workspace.version}');

    print(buffer.toString().trimRight());
    return 0;
  }
}
