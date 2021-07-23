const express = require('express');
const app = express();
app.get('/ping', (req, res) => {
  res.send("webclient server running");
});
app.use(express.static('dist'));
app.listen(80);