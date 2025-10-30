import 'dart:async';
import 'dart:io';

import 'package:path/path.dart' as p;

/// Describes the Forge workspace environment discovered from disk.
class WorkspaceContext {
  WorkspaceContext._(this.workspaceRoot, this.version);

  /// Absolute path to the repository root.
  final String workspaceRoot;

  /// Repository version read from [version.txt].
  final String version;

  static const _markerFiles = ['version.txt', 'melos.yaml'];

  /// Finds the workspace root by traversing up from [startDirectory].
  static Future<WorkspaceContext> find({Directory? startDirectory}) async {
    Directory directory = startDirectory ?? Directory.current;
    final visited = <String>{};

    while (true) {
      final normalized = p.normalize(directory.absolute.path);
      if (!visited.add(normalized)) {
        break; // Prevent potential cycles caused by symlinks.
      }

      if (await _containsMarkers(directory)) {
        final versionFile = File(p.join(normalized, 'version.txt'));
        if (!await versionFile.exists()) {
          throw WorkspaceContextException(
            'Found potential workspace at $normalized but version.txt missing.',
          );
        }

        final rawVersion = (await versionFile.readAsString()).trim();
        if (rawVersion.isEmpty) {
          throw WorkspaceContextException(
            'version.txt at $normalized is empty.',
          );
        }

        return WorkspaceContext._(normalized, rawVersion);
      }

      final parent = directory.parent;
      if (parent.path == directory.path) {
        break;
      }
      directory = parent;
    }

    throw const WorkspaceContextException(
      'Unable to locate Forge workspace. Ensure version.txt and melos.yaml exist '
      'at the repository root and re-run from within the project.',
    );
  }

  static Future<bool> _containsMarkers(Directory directory) async {
    final entries = await directory
        .list(followLinks: false)
        .map((entity) => p.basename(entity.path))
        .toSet();
    return _markerFiles.every(entries.contains);
  }
}

/// Error thrown when workspace discovery fails.
class WorkspaceContextException implements IOException {
  const WorkspaceContextException(this.message);

  final String message;

  @override
  String toString() => 'WorkspaceContextException: $message';
}
