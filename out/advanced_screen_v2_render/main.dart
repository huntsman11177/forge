MaterialApp(
  debugShowCheckedModeBanner: false,

  children: [
    Builder(
      children: [
        Scaffold(
          backgroundColor: primary,

          children: [
            AppBar(
              backgroundColor: primary,
              elevation: 0,

              children: [
                Container(
                  margin: null,

                  children: [
                    Icon(
                      arg0: Icons.menu,
                      color: Colors.white,
                      size: 30,

                    ),
                  ]
                ),
              ]
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,

              children: [
                Padding(
                  children: [
                    only(
                      left: 30,
                      top: 30,

                    ),
                    Text(
                      arg0: "Generation",

                      children: [
                        TextStyle(
                          color: Colors.white,
                          fontSize: 35,
                          fontWeight: FontWeight.bold,
                          letterSpacing: 2,

                        ),
                      ]
                    ),
                  ]
                ),
                Expanded(
                  children: [
                    Stack(
                      children: [
                        Container(
                          margin: null,

                          children: [
                            BoxDecoration(
                              borderRadius: null,
                              color: Colors.white,

                            ),
                          ]
                        ),
                        FadeTransition(
                          opacity: _animationBody,

                          children: [
                            SlideTransition(
                              position: null,

                              children: [
                                Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  mainAxisAlignment: MainAxisAlignment.end,

                                  children: [
                                    Container(
                                      alignment: Alignment.centerLeft,
                                      height: centralBoxSize,

                                      children: [
                                        Container(
                                          height: 70,
                                          margin: null,
                                          width: 120,

                                          children: [
                                            BoxDecoration(
                                              border: null,
                                              borderRadius: null,

                                            ),
                                            Row(
                                              mainAxisAlignment: MainAxisAlignment.start,

                                              children: [
                                                Container(
                                                  height: 80,
                                                  width: 70,

                                                  children: [
                                                    BoxDecoration(
                                                      borderRadius: null,
                                                      color: null,

                                                    ),
                                                    Icon(
                                                      arg0: Icons.attach_money,
                                                      color: Colors.white,
                                                      size: 40,

                                                    ),
                                                  ]
                                                ),
                                                Icon(
                                                  arg0: Icons.flash_on,
                                                  color: null,
                                                  size: 40,

                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                      ]
                                    ),
                                    Container(
                                      alignment: Alignment.centerLeft,
                                      height: centralBoxSize,
                                      padding: null,

                                      children: [
                                        Column(
                                          crossAxisAlignment: CrossAxisAlignment.start,

                                          children: [
                                            Text(
                                              arg0: null,

                                              children: [
                                                TextStyle(
                                                  color: primaryDark,
                                                  fontSize: 16,
                                                  fontWeight: FontWeight.bold,

                                                ),
                                              ]
                                            ),
                                            Text(
                                              arg0: null,

                                              children: [
                                                TextStyle(
                                                  color: primaryDark,
                                                  fontSize: 66,
                                                  fontWeight: FontWeight.bold,

                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                      ]
                                    ),
                                    Container(
                                      alignment: Alignment.centerLeft,
                                      height: centralBoxSize,
                                      padding: null,

                                      children: [
                                        Column(
                                          crossAxisAlignment: CrossAxisAlignment.start,

                                          children: [
                                            Text(
                                              arg0: null,

                                              children: [
                                                TextStyle(
                                                  color: accent,
                                                  fontSize: 16,
                                                  fontWeight: FontWeight.bold,

                                                ),
                                              ]
                                            ),
                                            Text(
                                              arg0: null,

                                              children: [
                                                TextStyle(
                                                  color: accent,
                                                  fontSize: 36,

                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                      ]
                                    ),
                                    Container(
                                      height: 100,
                                      margin: null,

                                      children: [
                                        Row(
                                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,

                                          children: [
                                            FlatButton(
                                              onPressed: () {},

                                              children: [
                                                Icon(
                                                  arg0: Icons.lightbulb_outline,
                                                  color: Colors.white,
                                                  size: 40,

                                                ),
                                              ]
                                            ),
                                            FlatButton(
                                              onPressed: () {},

                                              children: [
                                                Icon(
                                                  arg0: Icons.wb_sunny,
                                                  color: Colors.white,
                                                  size: 40,

                                                ),
                                              ]
                                            ),
                                            FlatButton(
                                              onPressed: () {},

                                              children: [
                                                Icon(
                                                  arg0: Icons.brightness_3,
                                                  color: Colors.white,
                                                  size: 40,

                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                        BoxDecoration(
                                          borderRadius: null,
                                          color: accent,

                                        ),
                                      ]
                                    ),
                                  ]
                                ),
                              ]
                            ),
                          ]
                        ),
                        FadeTransition(
                          opacity: _animationBulb,

                          children: [
                            SlideTransition(
                              position: null,

                              children: [
                                Align(
                                  alignment: Alignment.topRight,

                                  children: [
                                    Stack(
                                      children: [
                                        Align(
                                          alignment: Alignment.topRight,

                                          children: [
                                            Container(
                                              height: 80,
                                              margin: null,
                                              width: 90,

                                              children: [
                                                Container(
                                                  children: [
                                                    BoxDecoration(
                                                      borderRadius: null,
                                                      color: Colors.white,

                                                    ),
                                                  ]
                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                        Align(
                                          alignment: Alignment.topRight,

                                          children: [
                                            Container(
                                              width: 200,

                                              children: [
                                                ShaderMask(
                                                  shaderCallback: (Rect bounds) {return LinearGradient(begin: Alignment.topLeft, end: Alignment(-0.4, -0.8), stops: [0.0, 0.5, 0.5, 1], colors: [accentLight, accentLight, accent, accent], tileMode: TileMode.repeated).createShader(bounds);},

                                                  children: [
                                                    Icon(
                                                      arg0: Icons.lightbulb_outline,
                                                      color: Colors.white,
                                                      size: 350,

                                                    ),
                                                  ]
                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                        Align(
                                          alignment: Alignment.topRight,

                                          children: [
                                            Container(
                                              height: 130,
                                              margin: null,
                                              width: 130,

                                              children: [
                                                BoxDecoration(
                                                  borderRadius: null,
                                                  color: null,

                                                ),
                                              ]
                                            ),
                                          ]
                                        ),
                                      ]
                                    ),
                                  ]
                                ),
                              ]
                            ),
                          ]
                        ),
                      ]
                    ),
                  ]
                ),
              ]
            ),
          ]
        ),
      ]
    ),
  ]
)