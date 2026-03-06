import "./core.css";
import { near_native_setting } from "./AppSetter";

function App() {
  near_native_setting();
  return (
    <main>
      <h1>Welcome to Tauri + React</h1>
    </main>
  );
}

export default App;
