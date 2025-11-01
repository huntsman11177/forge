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
      expect(document['version'], '1.0.0');
      final screens = (document['screens'] as List<dynamic>)
          .cast<Map<String, dynamic>>();
      expect(screens, isNotEmpty);

      expect(
        screens.any(
          (screen) => containsWidget(screen['root'] as Map<String, dynamic>, 'Scaffold'),
        ),
        isTrue,
        reason: 'Expected Scaffold widget in parsed output',
      );

      expect(
        screens.any(
          (screen) => containsWidget(screen['root'] as Map<String, dynamic>, 'Center'),
        ),
        isTrue,
        reason: 'Expected Center widget in parsed output',
      );
    });

    test('parses advanced screen with nested widgets', () async {
      final filePath = p.join(Directory.current.path, 'test_fixtures', 'advanced_screen.dart');
      final json = await parser.parseScreenToJson(filePath);
      final document = jsonDecode(json) as Map<String, dynamic>;

      final screens = (document['screens'] as List<dynamic>)
          .cast<Map<String, dynamic>>();
      expect(screens.length, greaterThanOrEqualTo(1));

      bool anyContains(String widget) => screens.any(
            (screen) => containsWidget(screen['root'] as Map<String, dynamic>, widget),
          );

      expect(anyContains('Scaffold'), isTrue);
      expect(anyContains('AppBar'), isTrue);
      expect(anyContains('Text'), isTrue);
      expect(anyContains('ListTile'), isTrue);
    });

    test('classifies literals vs expressions for advanced_screen_v2', () async {
      final filePath = p.join(Directory.current.path, 'test_fixtures', 'advanced_screen_v2.dart');
      final document = await parser.parseScreen(filePath);

      final screens = (document['screens'] as List<dynamic>)
          .cast<Map<String, dynamic>>();
      expect(screens.length, equals(1));

      final root = screens.first['root'] as Map<String, dynamic>;
      final props = (root['props'] as Map<String, dynamic>);
      final debugBanner = props['debugShowCheckedModeBanner'] as Map<String, dynamic>;
      expect(debugBanner['type'], equals('literal'));
      expect(debugBanner['value'], isFalse);

      final builder = (root['children'] as List<dynamic>).first as Map<String, dynamic>;
      final builderProps = builder['props'] as Map<String, dynamic>;
      final builderClosure = builderProps['builder'] as Map<String, dynamic>;
      expect(builderClosure['type'], equals('expression'));
      expect(builderClosure['expression'], contains('MediaQuery.of'));
    });

    test('extracts logic callbacks for advanced screen', () async {
      final filePath = p.join(Directory.current.path, 'test_fixtures', 'advanced_screen.dart');
      final document = await parser.parseScreen(filePath);

      final logic = (document['logic'] as List<dynamic>?)?.cast<Map<String, dynamic>>() ?? [];
      expect(logic, isNotEmpty, reason: 'Expected logic nodes to be captured');

      bool containsEvent(String eventName) => logic.any((node) => node['event'] == eventName);

      expect(containsEvent('onChanged'), isTrue);
      expect(containsEvent('onTap'), isTrue);
    });

    test('serializes enums and colors as structured literals', () async {
      final filePath = p.join(Directory.current.path, 'test_fixtures', 'advanced_screen.dart');
      final document = await parser.parseScreen(filePath);

      final screens = (document['screens'] as List<dynamic>)
          .cast<Map<String, dynamic>>();
      final root = screens.first['root'] as Map<String, dynamic>;

      final backgroundColor = findProp(root, widget: 'AppBar', prop: 'backgroundColor');
      expect(backgroundColor, isNotNull);
      expect(backgroundColor!['type'], equals('literal'));
      expect(backgroundColor['value'], equals('#FF00BF6D'));

      final crossAxis = findProp(root, widget: 'Column', prop: 'crossAxisAlignment');
      expect(crossAxis, isNotNull);
      expect(crossAxis!['type'], equals('enum'));
      expect(crossAxis['value'], equals('CrossAxisAlignment.start'));
    });

    test('round-trips basic screen through schema validation', () async {
      final fixturePath = p.join(Directory.current.path, 'test_fixtures', 'basic_screen.dart');
      final document = await parser.parseScreen(fixturePath);

      final schemaPath = p.normalize(
        p.join(Directory.current.path, '..', '..', 'forge_spec', 'graph_schema.json'),
      );
      final results = await FlutterParser.validateGraphSchema(
        document,
        schemaPath: schemaPath,
      );

      expect(results.isValid, isTrue, reason: 'Expected schema validation to succeed');
    });
  });
}

bool containsWidget(Map<String, dynamic> node, String widgetName) {
  if (node['widget'] == widgetName) {
    return true;
  }
  final children = node['children'];
  if (children is List) {
    for (final child in children) {
      if (child is Map<String, dynamic> && containsWidget(child, widgetName)) {
        return true;
      }
    }
  }
  final props = node['props'];
  if (props is Map<String, dynamic>) {
    for (final value in props.values) {
      if (value is Map<String, dynamic> && containsWidget(value, widgetName)) {
        return true;
      }
      if (value is List) {
        for (final element in value) {
          if (element is Map<String, dynamic> && containsWidget(element, widgetName)) {
            return true;
          }
        }
      }
    }
  }
  return false;
}

Map<String, dynamic>? findProp(
  Map<String, dynamic> node, {
  required String widget,
  required String prop,
}) {
  if (node['widget'] == widget) {
    final props = node['props'];
    if (props is Map<String, dynamic>) {
      final value = props[prop];
      if (value is Map<String, dynamic>) {
        return value;
      }
    }
  }
  final children = node['children'];
  if (children is List) {
    for (final child in children) {
      if (child is Map<String, dynamic>) {
        final match = findProp(child, widget: widget, prop: prop);
        if (match != null) {
          return match;
        }
      }
    }
  }
  final props = node['props'];
  if (props is Map<String, dynamic>) {
    for (final value in props.values) {
      if (value is Map<String, dynamic>) {
        final match = findProp(value, widget: widget, prop: prop);
        if (match != null) {
          return match;
        }
      } else if (value is List) {
        for (final element in value) {
          if (element is Map<String, dynamic>) {
            final match = findProp(element, widget: widget, prop: prop);
            if (match != null) {
              return match;
            }
          }
        }
      }
    }
  }
  return null;
}
