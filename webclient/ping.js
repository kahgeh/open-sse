const express = require('express');
const app = express();
app.get('/', (req, res) => {
  res.send("webclient server running");
});
app.listen(8080);