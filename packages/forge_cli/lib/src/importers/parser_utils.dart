import 'dart:io';

import 'package:analyzer/dart/analysis/features.dart';
import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/error/error.dart';
import 'package:path/path.dart' as p;

/// Shared helpers for the Flutter reverse-import pipeline.
class ParserUtils {
  ParserUtils._();

  /// Reads the Dart file at [path] and returns the parsed compilation unit.
  /// Throws [FlutterParserIoError] when the file cannot be read, and
  /// [FlutterParserSyntaxError] when the analyzer reports errors.
  static CompilationUnit parseDartFile(String path) {
    final resolved = File(path);
    if (!resolved.existsSync()) {
      throw FlutterParserIoError('Dart file not found: ${resolved.path}');
    }

    final result = parseFile(
      path: resolved.path,
      featureSet: FeatureSet.latestLanguageVersion(),
    );
    final fatalErrors = result.errors
        .where((error) => error.errorCode.type == ErrorType.SYNTACTIC_ERROR)
        .toList();
    if (fatalErrors.isNotEmpty) {
      throw FlutterParserSyntaxError(resolved.path, fatalErrors);
    }

    return result.unit;
  }

  /// Returns `true` when [node] extends `StatelessWidget`.
  static bool isStatelessWidget(ClassDeclaration node) {
    final extendsClause = node.extendsClause;
    if (extendsClause == null) {
      return false;
    }
    final typeName = extendsClause.superclass.toSource();
    return typeName == 'StatelessWidget' ||
        typeName.endsWith('.StatelessWidget');
  }

  /// Convenience helper for checking whether a [TypeAnnotation] refers to
  /// Flutter's `Widget` type (either directly or imported via alias).
  static bool returnsWidget(TypeAnnotation? type) {
    if (type == null) {
      return false;
    }
    final source = type.toSource();
    return source == 'Widget' || source.endsWith('.Widget');
  }

  /// Generates a stable screen identifier based on the file path and class
  /// name. For now this is a simple lower_snake_case projection; future passes
  /// can hook into workspace metadata.
  static String extractScreenId(String filePath, String className) {
    final stem = p.basenameWithoutExtension(filePath);
    return '${_snakeCase(stem)}_${_snakeCase(className)}';
  }

  /// Returns the argument label if [argument] is a named expression.
  static String? argumentName(Expression argument) {
    return argument is NamedExpression ? argument.name.label.name : null;
  }

  /// Returns the underlying expression for [argument], unwrapping named
  /// arguments to their value expression.
  static Expression argumentValue(Expression argument) {
    return argument is NamedExpression ? argument.expression : argument;
  }

  static String _snakeCase(String value) {
    final buffer = StringBuffer();
    for (var i = 0; i < value.length; i++) {
      final char = value[i];
      if (char.toUpperCase() == char && i != 0) {
        buffer.write('_');
      }
      buffer.write(char.toLowerCase());
    }
    return buffer.toString();
  }
}

class FlutterParserIoError implements Exception {
  FlutterParserIoError(this.message);

  final String message;

  @override
  String toString() => 'FlutterParserIoError: $message';
}

class FlutterParserSyntaxError implements Exception {
  FlutterParserSyntaxError(this.filePath, this.errors);

  final String filePath;
  final List<AnalysisError> errors;

  @override
  String toString() {
    final joined = errors.map((e) => '${e.message} @ ${e.offset}').join('; ');
    return 'FlutterParserSyntaxError: $filePath -> $joined';
  }
}
