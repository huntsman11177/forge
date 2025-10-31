import 'dart:convert';

import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:path/path.dart' as p;

import 'parser_utils.dart';
import 'widget_map.dart';

class WidgetNode {
  WidgetNode(
    this.widget, {
    Map<String, dynamic>? props,
    List<WidgetNode>? children,
  })  : props = props ?? <String, dynamic>{},
        children = children ?? <WidgetNode>[];

  final String widget;
  final Map<String, dynamic> props;
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

    final projectId = _deriveProjectId(filePath);
    return {
      'forge_schema_version': '1.0.0',
      'project': {
        'id': projectId,
        'name': projectId,
      },
      'screens': collector.buildScreens(),
      'logic': <Object?>[],
    };
  }

  /// Convenience helper that returns a pretty-printed JSON representation of
  /// the parsed Forge schema document for [filePath].
  Future<String> parseScreenToJson(String filePath) async {
    final document = await parseScreen(filePath);
    return const JsonEncoder.withIndent('  ').convert(document);
  }
}

/// Visits AST nodes to discover widget entry points (e.g. StatelessWidget build
/// methods or `Widget buildHome()` helper functions).
class _WidgetCollector extends RecursiveAstVisitor<void> {
  _WidgetCollector(this.entryPointPath);

  final String entryPointPath;
  final List<Map<String, Object?>> _screens = [];

  @override
  void visitClassDeclaration(ClassDeclaration node) {
    if (ParserUtils.isStatelessWidget(node)) {
      final screenId = ParserUtils.extractScreenId(entryPointPath, node.name.lexeme);
      for (final member in node.members) {
        if (member is MethodDeclaration &&
            member.name.lexeme == 'build' &&
            ParserUtils.returnsWidget(member.returnType)) {
          final root = _extractRootWidget(member.body);
          // ignore: avoid_print
          print('Discovered build method for ${node.name.lexeme} with root: ${root?.widget}');
          _recordScreen(screenId, node.name.lexeme, root);
        }
      }
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

  void _recordScreen(String screenId, String screenName, WidgetNode? root) {
    if (root == null) {
      // ignore: avoid_print
      print('Skipping screen $screenId because root widget was null');
      return;
    }
    // ignore: avoid_print
    print('Recording screen $screenId with root ${root.widget}');
    _screens.add({
      'id': screenId,
      'name': screenName,
      'root': root.toJson(),
      'entry': false,
      'providers': <Object?>[],
    });
  }
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
  return null;
}

WidgetNode? _widgetFromInstance(InstanceCreationExpression expression) {
  final rawType = expression.constructorName.type.toSource();
  final simpleName = rawType.contains('.') ? rawType.split('.').last : rawType;
  final widgetType = WidgetCanonicalizer.canonicalize(simpleName);

  final props = <String, dynamic>{};
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
    props[key] = value;
  }

  return WidgetNode(widgetType, props: props, children: children);
}

dynamic _materializeValue(Expression expression) {
  if (expression is InstanceCreationExpression) {
    return _widgetFromInstance(expression);
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
String _deriveProjectId(String path) {
  return p.basenameWithoutExtension(path);
}
