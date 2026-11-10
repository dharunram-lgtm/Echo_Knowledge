import React from "react";
import Providers from "./providers";
import AppRouter from "./router";
import "./styles/globals.css";

  export const App: React.FC = () => {
    return (
    <Providers>
      <AppRouter />
    </Providers>
    );
};

export default App;
