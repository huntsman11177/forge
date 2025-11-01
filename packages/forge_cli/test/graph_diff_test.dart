import 'package:forge_cli/src/graph_diff.dart';
import 'package:test/test.dart';

void main() {
  group('GraphDiffer', () {
    test('detects added screens, prop changes, and logic differences', () {
      final baseline = {
        'version': '1.0.0',
        'screens': [
          {
            'id': 'screen_home',
            'name': 'HomeScreen',
            'root': {
              'widget': 'Scaffold',
              'props': {
                'backgroundColor': {'type': 'literal', 'value': '#FFFFFFFF'},
              },
              'children': <Map<String, dynamic>>[],
            },
          },
        ],
        'logic': [
          {
            'id': 'logic_0000',
            'type': 'callback',
            'screen': 'screen_home',
            'widgetPath': 'root/button',
            'widget': 'ElevatedButton',
            'event': 'onPressed',
            'expression': 'handleTap()',
          },
        ],
      };

      final candidate = {
        'version': '1.0.0',
        'screens': [
          {
            'id': 'screen_home',
            'name': 'HomeScreen',
            'root': {
              'widget': 'Scaffold',
              'props': {
                'backgroundColor': {'type': 'literal', 'value': '#FF000000'},
              },
              'children': <Map<String, dynamic>>[],
            },
          },
          {
            'id': 'screen_details',
            'name': 'DetailsScreen',
            'root': {
              'widget': 'Container',
              'props': <String, dynamic>{},
              'children': <Map<String, dynamic>>[],
            },
          },
        ],
        'logic': [
          {
            'id': 'logic_0000',
            'type': 'callback',
            'screen': 'screen_home',
            'widgetPath': 'root/button',
            'widget': 'ElevatedButton',
            'event': 'onPressed',
            'expression': 'handleTap()',
            'intents': ['navigation'],
            'metadata': {
              'navigation': {'route': '/details'},
            },
          },
          {
            'id': 'logic_0001',
            'type': 'callback',
            'screen': 'screen_details',
            'widgetPath': 'root',
            'widget': 'ListTile',
            'event': 'onTap',
            'expression': 'handleDetails()',
          },
        ],
      };

      final result = GraphDiffer.diff(baseline, candidate);

      expect(result.hasChanges, isTrue);
      expect(result.changes, contains('Screen added: screen_details (DetailsScreen)'));
      expect(
        result.changes,
        contains(
          '  • root prop "backgroundColor" changed: {"type":"literal","value":"#FFFFFFFF"} → {"type":"literal","value":"#FF000000"}',
        ),
      );
      expect(result.changes, contains('Logic added: logic_0001 (onTap)'));
      expect(result.changes, contains('Logic node logic_0000 changed:'));

      final diffDetails = result.toJson();
      final changedScreens = diffDetails['screens']['changed'] as Map<String, dynamic>;
      expect(changedScreens.containsKey('screen_home'), isTrue);
      final addedLogic = diffDetails['logic']['added'] as List<dynamic>;
      expect(addedLogic, contains('logic_0001'));
    });

    test('returns no changes when graphs are identical', () {
      final graph = {
        'version': '1.0.0',
        'screens': [
          {
            'id': 'screen_home',
            'name': 'HomeScreen',
            'root': {
              'widget': 'Scaffold',
              'props': <String, dynamic>{},
              'children': <Map<String, dynamic>>[],
            },
          },
        ],
        'logic': const [],
      };

      final result = GraphDiffer.diff(graph, graph);
      expect(result.hasChanges, isFalse);
      expect(result.changes, isEmpty);
      final details = result.toJson();
      expect(details['screens']['added'], isEmpty);
      expect(details['screens']['removed'], isEmpty);
      expect(details['logic']['added'], isEmpty);
      expect(details['logic']['removed'], isEmpty);
    });
  });
}
