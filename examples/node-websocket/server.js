require("uWebSockets.js")
  .App({})
  .ws("/*", {
    message: (ws, msg) => {
      ws.send(msg);
    },
  })
  .listen(9001, (s) => {
    if (s) {
      console.log("Listening to port 9001");
    }
  });
