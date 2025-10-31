import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:test/test.dart';

import 'package:forge_cli/src/importers/flutter_parser.dart';

void main() {
  group('FlutterParser', () {
    final parser = const FlutterParser();

    test('parses basic screen scaffold', () async {
      final filePath = p.join(Directory.current.path, 'test_fixtures', 'basic_screen.dart');
      final document = await parser.parseScreen(filePath);
      // Debug output for inspection during test development.
      // ignore: avoid_print
      print(jsonEncode(document));

      expect(document['forge_schema_version'], '1.0.0');
      final screens = document['screens'] as List<dynamic>;
      expect(screens, isNotEmpty);

      final screen = screens.first as Map<String, dynamic>;
      expect(screen['id'], isNotEmpty);
      expect(screen['root'], isA<Map<String, dynamic>>());

      final root = screen['root'] as Map<String, dynamic>;
      expect(root['widget'], 'Scaffold');

      final props = root['props'] as Map<String, dynamic>;
      expect(props.containsKey('appBar'), isTrue);
      expect(props['appBar'], isA<Map<String, dynamic>>());

      final children = root['children'] as List<dynamic>;
      expect(children, isNotEmpty);
      final body = children.first as Map<String, dynamic>;
      expect(body['widget'], 'Center');
    });

    test('parses advanced screen with nested widgets', () async {
      final filePath = p.join(Directory.current.path, 'test_fixtures', 'advanced_screen.dart');
      final json = await parser.parseScreenToJson(filePath);
      // ignore: avoid_print
      print(json);
      final document = jsonDecode(json) as Map<String, dynamic>;

      final screens = document['screens'] as List<dynamic>;
      expect(screens.length, greaterThanOrEqualTo(1));

      final screen = screens.first as Map<String, dynamic>;
      final root = screen['root'] as Map<String, dynamic>;
      expect(root['widget'], 'Scaffold');

      final props = root['props'] as Map<String, dynamic>;
      expect(props['appBar'], isNotNull);
      expect((props['appBar'] as Map<String, dynamic>)['widget'], 'AppBar');

      final rootChildren = (root['children'] as List<dynamic>).cast<Map<String, dynamic>>();
      expect(rootChildren, isNotEmpty);

      final column = rootChildren.firstWhere(
        (child) => child['widget'] == 'Column',
        orElse: () => throw StateError('Column child not found'),
      );

      final columnChildren = (column['children'] as List<dynamic>).cast<Map<String, dynamic>>();
      expect(columnChildren.length, greaterThan(1));
    });
  });
}
