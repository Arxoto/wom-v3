import "./core.css";
import { set_page_config } from "./core";

const App = () => {
  set_page_config();
  return (
    <main>
      <h1>Welcome to Tauri + React</h1>
    </main>
  );
}

export default App;
