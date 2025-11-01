import 'dart:convert';

class GraphDiffResult {
  GraphDiffResult({required this.changes, required this.json});

  final List<String> changes;
  final Map<String, dynamic> json;

  bool get hasChanges => changes.isNotEmpty;

  Map<String, dynamic> toJson() => json;
}

class GraphDiffer {
  static GraphDiffResult diff(
    Map<String, dynamic> baseline,
    Map<String, dynamic> candidate,
  ) {
    final changes = <String>[];
    final json = <String, dynamic>{
      'version': {},
      'screens': {
        'added': <String>[],
        'removed': <String>[],
        'changed': <String, dynamic>{},
      },
      'logic': {
        'added': <String>[],
        'removed': <String>[],
        'changed': <String, dynamic>{},
      },
    };

    if (baseline['version'] != candidate['version']) {
      changes.add(
        'Version changed: ${baseline['version']} → ${candidate['version']}',
      );
      json['version'] = {
        'baseline': baseline['version'],
        'candidate': candidate['version'],
      };
    }

    _diffScreens(baseline, candidate, changes, json['screens']);
    _diffLogic(baseline, candidate, changes, json['logic']);

    return GraphDiffResult(changes: changes, json: json);
  }

  static void _diffScreens(
    Map<String, dynamic> baseline,
    Map<String, dynamic> candidate,
    List<String> changes,
    Map<String, dynamic> json,
  ) {
    final baselineScreens = _indexById(
      (baseline['screens'] as List<dynamic>? ?? <dynamic>[])
          .cast<Map<String, dynamic>>(),
    );
    final candidateScreens = _indexById(
      (candidate['screens'] as List<dynamic>? ?? <dynamic>[])
          .cast<Map<String, dynamic>>(),
    );

    final removed = baselineScreens.keys.toSet()
      ..removeWhere(candidateScreens.containsKey);
    final added = candidateScreens.keys.toSet()
      ..removeWhere(baselineScreens.containsKey);

    if (removed.isNotEmpty) {
      for (final id in removed) {
        changes.add('Screen removed: $id');
      }
      (json['removed'] as List<String>).addAll(removed);
    }

    if (added.isNotEmpty) {
      for (final id in added) {
        final screen = candidateScreens[id]!;
        final name = screen['name'];
        changes.add('Screen added: $id${name != null ? ' ($name)' : ''}');
      }
      (json['added'] as List<String>).addAll(added);
    }

    final shared = baselineScreens.keys
        .toSet()
      ..retainWhere(candidateScreens.containsKey);

    final changedScreens = json['changed'] as Map<String, dynamic>;

    for (final id in shared) {
      final baselineScreen = baselineScreens[id]!;
      final candidateScreen = candidateScreens[id]!;
      final widgetDiffs = _diffWidget(
        baselineScreen['root'] as Map<String, dynamic>,
        candidateScreen['root'] as Map<String, dynamic>,
        'root',
      );
      if (widgetDiffs.isNotEmpty) {
        changes.add('Screen $id changed:');
        for (final diff in widgetDiffs) {
          changes.add('  • $diff');
        }
        changedScreens[id] = widgetDiffs;
      }
    }
  }

  static void _diffLogic(
    Map<String, dynamic> baseline,
    Map<String, dynamic> candidate,
    List<String> changes,
    Map<String, dynamic> json,
  ) {
    final baselineLogic = _indexById(
      (baseline['logic'] as List<dynamic>? ?? <dynamic>[])
          .cast<Map<String, dynamic>>(),
    );
    final candidateLogic = _indexById(
      (candidate['logic'] as List<dynamic>? ?? <dynamic>[])
          .cast<Map<String, dynamic>>(),
    );

    final removed = baselineLogic.keys.toSet()
      ..removeWhere(candidateLogic.containsKey);
    final added = candidateLogic.keys.toSet()
      ..removeWhere(baselineLogic.containsKey);

    if (removed.isNotEmpty) {
      for (final id in removed) {
        changes.add('Logic removed: $id');
      }
      (json['removed'] as List<String>).addAll(removed);
    }

    if (added.isNotEmpty) {
      for (final id in added) {
        final node = candidateLogic[id]!;
        changes.add('Logic added: $id (${node['event'] ?? 'unknown event'})');
      }
      (json['added'] as List<String>).addAll(added);
    }

    final shared = baselineLogic.keys
        .toSet()
      ..retainWhere(candidateLogic.containsKey);

    final changedLogic = json['changed'] as Map<String, dynamic>;

    for (final id in shared) {
      final baselineNode = baselineLogic[id]!;
      final candidateNode = candidateLogic[id]!;
      final nodeChanges = <String>[];

      void compareField(String field) {
        if (!_deepEquals(baselineNode[field], candidateNode[field])) {
          nodeChanges.add(
            '$field changed: ${_stringify(baselineNode[field])} → ${_stringify(candidateNode[field])}',
          );
        }
      }

      compareField('event');
      compareField('widgetPath');
      compareField('widget');
      compareField('intents');
      compareField('expression');
      compareField('metadata');

      if (nodeChanges.isNotEmpty) {
        changes.add('Logic node $id changed:');
        for (final diff in nodeChanges) {
          changes.add('  • $diff');
        }
        changedLogic[id] = nodeChanges;
      }
    }
  }

  static Map<String, Map<String, dynamic>> _indexById(
    List<Map<String, dynamic>> nodes,
  ) {
    final map = <String, Map<String, dynamic>>{};
    for (final node in nodes) {
      final id = node['id'];
      if (id is String && id.isNotEmpty) {
        map[id] = node;
      }
    }
    return map;
  }

  static List<String> _diffWidget(
    Map<String, dynamic> baseline,
    Map<String, dynamic> candidate,
    String path,
  ) {
    final diffs = <String>[];

    if (baseline['widget'] != candidate['widget']) {
      diffs.add(
        '$path widget changed: ${baseline['widget']} → ${candidate['widget']}',
      );
    }

    final baselineProps = (baseline['props'] as Map<String, dynamic>? ?? {});
    final candidateProps = (candidate['props'] as Map<String, dynamic>? ?? {});
    final propKeys = <String>{}
      ..addAll(baselineProps.keys)
      ..addAll(candidateProps.keys);

    for (final key in propKeys) {
      final baselineValue = baselineProps[key];
      final candidateValue = candidateProps[key];

      if (!_deepEquals(baselineValue, candidateValue)) {
        diffs.add(
          '$path prop "$key" changed: ${_stringify(baselineValue)} → ${_stringify(candidateValue)}',
        );
      }
    }

    final baselineChildren =
        (baseline['children'] as List<dynamic>? ?? const <dynamic>[])
            .cast<Map<String, dynamic>>();
    final candidateChildren =
        (candidate['children'] as List<dynamic>? ?? const <dynamic>[])
            .cast<Map<String, dynamic>>();

    final minLength = baselineChildren.length < candidateChildren.length
        ? baselineChildren.length
        : candidateChildren.length;

    for (var i = 0; i < minLength; i++) {
      diffs.addAll(
        _diffWidget(
          baselineChildren[i],
          candidateChildren[i],
          '$path/children[$i]',
        ),
      );
    }

    if (candidateChildren.length > baselineChildren.length) {
      for (var i = baselineChildren.length; i < candidateChildren.length; i++) {
        diffs.add('$path child added at index $i');
      }
    } else if (baselineChildren.length > candidateChildren.length) {
      for (var i = candidateChildren.length; i < baselineChildren.length; i++) {
        diffs.add('$path child removed at index $i');
      }
    }

    return diffs;
  }

  static bool _deepEquals(dynamic a, dynamic b) {
    if (identical(a, b)) {
      return true;
    }
    if (a == null || b == null) {
      return a == b;
    }
    if (a is Map && b is Map) {
      if (a.length != b.length) {
        return false;
      }
      for (final key in a.keys) {
        if (!b.containsKey(key)) {
          return false;
        }
        if (!_deepEquals(a[key], b[key])) {
          return false;
        }
      }
      return true;
    }
    if (a is List && b is List) {
      if (a.length != b.length) {
        return false;
      }
      for (var i = 0; i < a.length; i++) {
        if (!_deepEquals(a[i], b[i])) {
          return false;
        }
      }
      return true;
    }
    return a == b;
  }

  static String _stringify(dynamic value) {
    if (value == null) {
      return 'null';
    }
    if (value is String) {
      return '"$value"';
    }
    if (value is Map || value is List) {
      return jsonEncode(value);
    }
    return value.toString();
  }
}
