import { HashRouter, Routes, Route, useLocation } from "react-router-dom";
import { AnimatePresence } from "framer-motion";
import Dashboard from "./components/Dashboard";
import Wallet from "./pages/Wallet";
import Network from "./pages/Network";
import Transactions from "./pages/Transactions";
import Mempool from "./pages/Mempool";
import Settings from "./pages/Settings";
import Explorer from "./pages/Explorer";
import Tokenomics from "./pages/Tokenomics";
import { ThemeProvider } from "./context/ThemeContext";
import { AppProvider } from "./context/AppContext";
import { ToastProvider } from "./context/ToastContext";
import Layout from "./components/Layout";
import { WelcomeAnimation } from "./components/WelcomeAnimation";
import "./App.css";

function AnimatedRoutes() {
  const location = useLocation();

  return (
    <AnimatePresence mode="wait">
      <Routes location={location} key={location.pathname}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/wallet" element={<Wallet />} />
        <Route path="/network" element={<Network />} />
        <Route path="/tokenomics" element={<Tokenomics />} />
        <Route path="/explorer" element={<Explorer />} />
        <Route path="/transactions" element={<Transactions />} />
        <Route path="/mempool" element={<Mempool />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </AnimatePresence>
  );
}

function App() {
  return (
    <ThemeProvider>
      <ToastProvider>
        <AppProvider>
          <HashRouter>
            <Layout>
              <WelcomeAnimation />
              <AnimatedRoutes />
            </Layout>
          </HashRouter>
        </AppProvider>
      </ToastProvider>
    </ThemeProvider>
  );
}

export default App;
