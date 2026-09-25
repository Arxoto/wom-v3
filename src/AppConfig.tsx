import "./core.css";
import { setup_page_config } from "./core";

const App = () => {
  setup_page_config();
  return (
    <main>
      <h1>Welcome to Tauri + React</h1>
    </main>
  );
}

export default App;
