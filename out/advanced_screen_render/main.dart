Scaffold(
  backgroundColor: Colors.white,

  children: [
    AppBar(
      backgroundColor: "#FF00BF6D",
      centerTitle: false,
      elevation: 0,
      foregroundColor: Colors.white,

      children: [
        Text(
          arg0: "People",

        ),
      ]
    ),
    Column(
      children: [
        Container(
          color: "#FF00BF6D",

          children: [
            only(
              bottom: 16.0,

            ),
            fromLTRB(
              arg0: 16.0,
              arg1: 0,
              arg2: 16.0,
              arg3: 16.0,

            ),
            Form(
              children: [
                TextFormField(
                  autofocus: true,
                  onChanged: (value) {},
                  textInputAction: TextInputAction.search,

                  children: [
                    InputDecoration(
                      fillColor: Colors.white,
                      filled: true,
                      hintText: "Search",

                      children: [
                        Icon(
                          arg0: Icons.search,
                          color: null,

                        ),
                        TextStyle(
                          color: null,

                        ),
                        symmetric(
                          horizontal: 16.0 * 1.5,
                          vertical: 16.0,

                        ),
                        OutlineInputBorder(
                          borderRadius: null,
                          borderSide: BorderSide.none,

                        ),
                      ]
                    ),
                  ]
                ),
              ]
            ),
          ]
        ),
        Expanded(
          children: [
            SingleChildScrollView(
              children: [
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,

                  children: [
                    RecentSearchContacts(
                    ),
                    SizedBox(
                      height: 16.0,

                    ),
                    Padding(
                      children: [
                        symmetric(
                          horizontal: 16.0,

                        ),
                        Text(
                          arg0: "Phone contacts",
                          style: null,

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