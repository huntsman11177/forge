import 'package:flutter/material.dart';

class BasicScreen extends StatelessWidget {
  const BasicScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Forge Demo'),
      ),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: const [
            Text('Hello Forge!'),
            SizedBox(height: 12),
            ElevatedButton(
              onPressed: null,
              child: Text('Press'),
            ),
          ],
        ),
      ),
    );
  }
}
