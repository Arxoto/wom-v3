import { Box, Static, Elastic, Divider } from "./main/Layout";
import { Head } from "./main/Head";

import "./core.css";

function App() {
  return (
    <Box>
      <Static>
        <Head></Head>
      </Static>
      <Divider></Divider>
      <Elastic>
        asdsdadadadasdasd
        <br />
        asdsdadadadasdasd
        <br />
      </Elastic>
      <Divider></Divider>
      <Static>
        asdsadsad
      </Static>
    </Box>
  );
}

export default App;
