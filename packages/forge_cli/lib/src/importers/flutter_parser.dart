import 'dart:convert';
import 'dart:io';

import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:json_schema/json_schema.dart';
import 'package:path/path.dart' as p;

import 'parser_utils.dart';
import 'widget_map.dart';

class WidgetNode {
  WidgetNode(
    this.widget, {
    Map<String, Map<String, dynamic>>? props,
    List<WidgetNode>? children,
  })  : props = props ?? <String, Map<String, dynamic>>{},
        children = children ?? <WidgetNode>[];

  final String widget;
  final Map<String, Map<String, dynamic>> props;
  final List<WidgetNode> children;

  Map<String, dynamic> toJson() => {
        'widget': widget,
        'props': props,
        if (children.isNotEmpty)
          'children': children.map((child) => child.toJson()).toList(),
      };
}

/// Parses Flutter widget source files into Forge schema-friendly JSON payloads.
class FlutterParser {
  const FlutterParser();

  /// Parses [filePath] and returns a Forge schema document containing screen
  /// graphs derived from the Dart AST.
  Future<Map<String, dynamic>> parseScreen(String filePath) async {
    final unit = ParserUtils.parseDartFile(filePath);
    final collector = _WidgetCollector(filePath);
    unit.accept(collector);

    final screens = collector.buildScreens();
    final canonicalScreens = <Map<String, Object?>>[];
    for (var index = 0; index < screens.length; index++) {
      final screen = Map<String, Object?>.from(screens[index]);
      screen['entry'] = index == 0;
      canonicalScreens.add(screen);
    }

    final projectId = p.basenameWithoutExtension(filePath);

    final result = <String, Object?>{
      'forge_schema_version': '1.0.0',
      'project': {
        'id': projectId,
        'name': projectId,
      },
      'screens': canonicalScreens,
    };

    final logic = collector.buildLogic();
    if (logic.isNotEmpty) {
      result['logic'] = logic;
    }
    return result;
  }

  /// Convenience helper that returns a pretty-printed JSON representation of
  /// the parsed Forge schema document for [filePath].
  Future<String> parseScreenToJson(String filePath) async {
    final document = await parseScreen(filePath);
    return const JsonEncoder.withIndent('  ').convert(document);
  }

  static Future<ValidationResults> validateGraphSchema(
    Map<String, dynamic> graph, {
    required String schemaPath,
  }) async {
    final schemaFile = File(schemaPath);
    if (!await schemaFile.exists()) {
      throw FlutterParserSchemaError('Schema not found at $schemaPath');
    }

    final schemaContent = await schemaFile.readAsString();
    final schemaJson = jsonDecode(schemaContent) as Map<String, dynamic>;
    final schema = await JsonSchema.createAsync(schemaJson);
    return schema.validate(graph);
  }
}

/// Visits AST nodes to discover widget entry points (e.g. StatelessWidget build
/// methods or `Widget buildHome()` helper functions).
class _WidgetCollector extends RecursiveAstVisitor<void> {
  _WidgetCollector(this.entryPointPath);

  final String entryPointPath;
  final List<Map<String, Object?>> _screens = [];
  final Set<String> _seenScreenIds = {};
  final List<Map<String, Object?>> _logic = [];
  int _logicCounter = 0;

  @override
  void visitClassDeclaration(ClassDeclaration node) {
    if (ParserUtils.isStatelessWidget(node)) {
      _collectBuildForClass(node, node.name.lexeme);
    } else if (ParserUtils.isStateClass(node)) {
      final widgetName = ParserUtils.widgetNameFromState(node) ??
          node.name.lexeme.replaceFirst(RegExp('^_'), '');
      _collectBuildForClass(node, widgetName);
    }
    super.visitClassDeclaration(node);
  }

  @override
  void visitFunctionDeclaration(FunctionDeclaration node) {
    if (ParserUtils.returnsWidget(node.returnType)) {
      final screenId = ParserUtils.extractScreenId(entryPointPath, node.name.lexeme);
      final root = _extractRootWidget(node.functionExpression.body);
      _recordScreen(screenId, node.name.lexeme, root);
    }
    super.visitFunctionDeclaration(node);
  }

  List<Map<String, Object?>> buildScreens() {
    return List<Map<String, Object?>>.unmodifiable(_screens);
  }

  List<Map<String, Object?>> buildLogic() {
    return List<Map<String, Object?>>.unmodifiable(_logic);
  }

  void _collectBuildForClass(ClassDeclaration node, String screenName) {
    final screenId = ParserUtils.extractScreenId(entryPointPath, screenName);
    for (final member in node.members) {
      if (member is MethodDeclaration && member.name.lexeme == 'build') {
        final returnType = member.returnType;
        if (returnType != null && !ParserUtils.returnsWidget(returnType)) {
          continue;
        }
        final root = _extractRootWidget(member.body);
        _recordScreen(screenId, screenName, root);
      }
    }
  }

  void _recordScreen(String screenId, String screenName, WidgetNode? root) {
    if (root == null) {
      return;
    }
    if (_seenScreenIds.contains(screenId)) {
      return;
    }
    _seenScreenIds.add(screenId);
    _collectLogicNodes(screenId, root, 'root');
    _screens.add({
      'id': screenId,
      'name': screenName,
      'root': root.toJson(),
      'entry': false,
      'providers': <Object?>[],
    });
  }

  void _collectLogicNodes(String screenId, WidgetNode node, String path) {
    node.props.forEach((propName, propValue) {
      final type = propValue['type'];
      if (type == 'expression') {
        final expression = propValue['expression'];
        if (expression is String &&
            _isEventProperty(propName) &&
            _looksLikeCallbackExpression(expression)) {
          final entry = <String, Object?>{
            'id': _nextLogicId(),
            'type': 'callback',
            'screen': screenId,
            'widgetPath': path,
            'widget': node.widget,
            'event': propName,
            'expression': expression,
          };
          final intents = <String>[];
          final metadata = <String, Object?>{};
          if (_looksLikeNavigation(expression)) {
            intents.add('navigation');
            metadata['navigation'] = _extractNavigationMetadata(expression);
          }
          if (_looksLikeAsync(expression)) {
            intents.add('async');
          }
          if (_looksLikeProviderUse(expression)) {
            intents.add('provider');
          }
          if (intents.isNotEmpty) {
            entry['intents'] = intents;
          }
          if (metadata.isNotEmpty) {
            entry['metadata'] = metadata;
          }
          _logic.add(entry);
        }
      }
    });

    for (var index = 0; index < node.children.length; index++) {
      final child = node.children[index];
      _collectLogicNodes(screenId, child, '$path/$index');
    }
  }

  String _nextLogicId() {
    final id = 'logic_${_logicCounter.toString().padLeft(4, '0')}';
    _logicCounter += 1;
    return id;
  }
}

bool _looksLikeCallbackExpression(String expression) {
  final trimmed = expression.trimLeft();
  if (trimmed.startsWith('(')) {
    return true;
  }
  if (trimmed.startsWith('async ')) {
    return true;
  }
  if (trimmed.startsWith('function')) {
    return true;
  }
  return _identifierPattern.hasMatch(trimmed);
}

bool _isEventProperty(String name) {
  if (!name.startsWith('on') || name.length < 3) {
    return false;
  }
  final third = name[2];
  return third.toUpperCase() == third;
}

bool _looksLikeNavigation(String expression) {
  const navigationHints = [
    'Navigator.push',
    'Navigator.of',
    'Navigator.pop',
    'GoRouter.of',
    'context.go',
    'context.push',
    'Navigator.pushNamed',
    'Navigator.pushReplacement',
  ];
  return navigationHints.any(expression.contains);
}

Map<String, Object?> _extractNavigationMetadata(String expression) {
  final metadata = <String, Object?>{};
  final routeMatch = RegExp(r'''['"]([^'"]+)['"]''').firstMatch(expression);
  if (routeMatch != null) {
    metadata['route'] = routeMatch.group(1);
  }
  if (expression.contains('pushReplacement')) {
    metadata['replace'] = true;
  }
  if (expression.contains('Navigator.pop')) {
    metadata['pop'] = true;
  }
  return metadata;
}

bool _looksLikeAsync(String expression) {
  return expression.contains('await ') || expression.contains('async');
}

bool _looksLikeProviderUse(String expression) {
  const providerHints = [
    'Provider.of',
    'context.read',
    'context.watch',
    'ref.read',
    'ref.watch',
    'Riverpod',
  ];
  return providerHints.any(expression.contains);
}

class _ExpressionValue {
  _ExpressionValue(this.source);

  final String source;
}

class _ColorValue {
  _ColorValue(this.argb);

  final int argb;

  String get hexString =>
      '#${argb.toUnsigned(32).toRadixString(16).padLeft(8, '0').toUpperCase()}';
}

WidgetNode? _extractRootWidget(FunctionBody body) {
  if (body is ExpressionFunctionBody) {
    return _widgetFromExpression(body.expression);
  }
  if (body is BlockFunctionBody) {
    for (final statement in body.block.statements) {
      if (statement is ReturnStatement) {
        return _widgetFromExpression(statement.expression);
      }
    }
  }
  return null;
}

WidgetNode? _widgetFromExpression(Expression? expression) {
  if (expression is InstanceCreationExpression) {
    return _widgetFromInstance(expression);
  }
  if (expression is MethodInvocation) {
    return _widgetFromInvocation(expression);
  }
  return null;
}

WidgetNode? _widgetFromInstance(InstanceCreationExpression expression) {
  final rawType = expression.constructorName.type.toSource();
  final simpleName = rawType.contains('.') ? rawType.split('.').last : rawType;
  final widgetType = WidgetCanonicalizer.canonicalize(simpleName);

  final props = <String, Map<String, dynamic>>{};
  final children = <WidgetNode>[];

  for (final argument in expression.argumentList.arguments) {
    final name = ParserUtils.argumentName(argument);
    final valueExpression = ParserUtils.argumentValue(argument);
    final value = _materializeValue(valueExpression);

    switch (name) {
      case 'child':
        if (value is WidgetNode) {
          children.add(value);
          continue;
        }
        break;
      case 'children':
        if (value is List<WidgetNode>) {
          children.addAll(value);
          continue;
        }
        break;
    }

    if (value is WidgetNode) {
      children.add(value);
      continue;
    }
    if (value is List<WidgetNode>) {
      children.addAll(value);
      continue;
    }

    final key = name ?? 'arg${props.length}';
    props[key] = _propValue(value);
  }

  return WidgetNode(widgetType, props: props, children: children);
}

dynamic _materializeValue(Expression expression) {
  if (expression is InstanceCreationExpression) {
    final colorLiteral = _tryParseColor(expression);
    if (colorLiteral != null) {
      return colorLiteral;
    }
    return _widgetFromInstance(expression);
  }
  if (expression is MethodInvocation) {
    return _widgetFromInvocation(expression);
  }
  if (expression is FunctionExpression) {
    final root = _extractRootWidget(expression.body);
    if (root != null) {
      return root;
    }
    return _ExpressionValue(expression.toSource());
  }
  if (expression is SimpleIdentifier) {
    return _ExpressionValue(expression.toSource());
  }
  if (expression is ListLiteral) {
    final values = <dynamic>[];
    for (final element in expression.elements) {
      if (element is Expression) {
        values.add(_materializeValue(element));
      }
    }
    if (values.isNotEmpty && values.every((value) => value is WidgetNode)) {
      return values.cast<WidgetNode>();
    }
    return values;
  }
  if (expression is SimpleStringLiteral) {
    return expression.value;
  }
  if (expression is IntegerLiteral) {
    return expression.value;
  }
  if (expression is DoubleLiteral) {
    return expression.value;
  }
  if (expression is BooleanLiteral) {
    return expression.value;
  }
  if (expression is NullLiteral) {
    return null;
  }
  return expression.toSource();
}

WidgetNode? _widgetFromInvocation(MethodInvocation invocation) {
  // Skip invocations that clearly are not widget constructors.
  if (invocation.target != null) {
    return null;
  }

  final methodName = invocation.methodName.name;
  if (methodName.isEmpty || !_startsWithUppercase(methodName)) {
    return null;
  }

  final widgetType = WidgetCanonicalizer.canonicalize(methodName);
  final props = <String, Map<String, dynamic>>{};
  final children = <WidgetNode>[];

  for (final argument in invocation.argumentList.arguments) {
    final name = ParserUtils.argumentName(argument);
    final valueExpression = ParserUtils.argumentValue(argument);
    final value = _materializeValue(valueExpression);

    switch (name) {
      case 'child':
        if (value is WidgetNode) {
          children.add(value);
          continue;
        }
        break;
      case 'children':
        if (value is List<WidgetNode>) {
          children.addAll(value);
          continue;
        }
        break;
    }

    if (value is WidgetNode) {
      children.add(value);
      continue;
    }
    if (value is List<WidgetNode>) {
      children.addAll(value);
      continue;
    }

    final key = name ?? 'arg${props.length}';
    props[key] = _propValue(value);
  }

  return WidgetNode(widgetType, props: props, children: children);
}

bool _startsWithUppercase(String value) {
  if (value.isEmpty) {
    return false;
  }
  final first = value.codeUnitAt(0);
  return first >= 0x41 && first <= 0x5A;
}

Map<String, dynamic> _propValue(dynamic value) {
  if (value is _ColorValue) {
    return {
      'type': 'literal',
      'value': value.hexString,
    };
  }
  if (value is _ExpressionValue) {
    return {
      'type': 'expression',
      'expression': value.source,
    };
  }
  final classified = _classifyValue(value);
  switch (classified.$1) {
    case _PropValueType.literal:
      return {
        'type': 'literal',
        'value': classified.$2,
      };
    case _PropValueType.enumValue:
      return {
        'type': 'enum',
        'value': classified.$2,
      };
    case _PropValueType.expression:
      return {
        'type': 'expression',
        'expression': classified.$2,
      };
  }
}

(_PropValueType, dynamic) _classifyValue(dynamic value) {
  if (value is _ColorValue) {
    return (_PropValueType.literal, value.hexString);
  }
  if (value is _ExpressionValue) {
    return (_PropValueType.expression, value.source);
  }
  if (value == null || value is num || value is bool) {
    return (_PropValueType.literal, value);
  }
  if (value is String) {
    if (_isEnumCandidate(value)) {
      return (_PropValueType.enumValue, value);
    }
    if (_looksLikeExpression(value)) {
      return (_PropValueType.expression, value);
    }
    return (_PropValueType.literal, value);
  }
  return (_PropValueType.expression, value.toString());
}

bool _isEnumCandidate(String value) {
  return _enumPattern.hasMatch(value);
}

bool _looksLikeExpression(String value) {
  const expressionHints = ['=>', '??', '??=', '?.', '::', 'return ', ' if ', ' else '];
  if (expressionHints.any((hint) => value.contains(hint))) {
    return true;
  }
  const operatorChars = ['(', ')', '{', '}', '[', ']', '=', '+', '-', '*', '/', '?', ':'];
  if (operatorChars.any((char) => value.contains(char))) {
    return true;
  }
  return false;
}

final RegExp _enumPattern = RegExp(r'^[A-Z][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+$');
final RegExp _identifierPattern = RegExp(r'^[A-Za-z_][A-Za-z0-9_]*$');

enum _PropValueType { literal, enumValue, expression }

_ColorValue? _tryParseColor(InstanceCreationExpression expression) {
  final typeSource = expression.constructorName.type.toSource();
  if (!(typeSource == 'Color' || typeSource.endsWith('.Color'))) {
    return null;
  }

  final constructorName = expression.constructorName.name?.name;
  final args = expression.argumentList.arguments;

  int? argb;
  if (constructorName == null) {
    if (args.length != 1) {
      return null;
    }
    final value = _readInt(args.first);
    if (value == null) {
      return null;
    }
    argb = value & 0xFFFFFFFF;
  } else if (constructorName == 'fromARGB') {
    if (args.length != 4) {
      return null;
    }
    final a = _readInt(args[0]);
    final r = _readInt(args[1]);
    final g = _readInt(args[2]);
    final b = _readInt(args[3]);
    if (a == null || r == null || g == null || b == null) {
      return null;
    }
    argb = ((a & 0xFF) << 24) |
        ((r & 0xFF) << 16) |
        ((g & 0xFF) << 8) |
        (b & 0xFF);
  } else if (constructorName == 'fromRGBO') {
    if (args.length != 4) {
      return null;
    }
    final r = _readInt(args[0]);
    final g = _readInt(args[1]);
    final b = _readInt(args[2]);
    final opacity = _readDouble(args[3]);
    if (r == null || g == null || b == null || opacity == null) {
      return null;
    }
    var alpha = (opacity * 255).round();
    if (alpha < 0) {
      alpha = 0;
    } else if (alpha > 255) {
      alpha = 255;
    }
    argb = ((alpha & 0xFF) << 24) |
        ((r & 0xFF) << 16) |
        ((g & 0xFF) << 8) |
        (b & 0xFF);
  } else {
    return null;
  }

  return _ColorValue(argb);
}

int? _readInt(Expression expression) {
  if (expression is IntegerLiteral) {
    return expression.value;
  }
  if (expression is PrefixExpression && expression.operator.lexeme == '-' &&
      expression.operand is IntegerLiteral) {
    final literal = expression.operand as IntegerLiteral;
    final value = literal.value;
    return value != null ? -value : null;
  }
  return null;
}

double? _readDouble(Expression expression) {
  if (expression is DoubleLiteral) {
    return expression.value;
  }
  if (expression is IntegerLiteral) {
    final value = expression.value;
    return value?.toDouble();
  }
  return null;
}

class FlutterParserSchemaError implements Exception {
  FlutterParserSchemaError(this.message);

  final String message;

  @override
  String toString() => 'FlutterParserSchemaError: $message';
}
