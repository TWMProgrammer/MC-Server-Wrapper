import { useState, useEffect } from 'react';
import { Minus, Square, Copy, X, Bug, RefreshCw, Users, Github } from 'lucide-react';
import { useAppSettings } from '../hooks/useAppSettings';
import './TitleBar.css';

export function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const { appVersion } = useAppSettings();
  const [appWindow, setAppWindow] = useState<any>(null);

  useEffect(() => {
    let unlisten: any;

    const initWindow = async () => {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      setAppWindow(win);

      const maximized = await win.isMaximized();
      setIsMaximized(maximized);

      // Listen for resize events to update maximization state
      unlisten = await win.onResized(() => {
        win.isMaximized().then(setIsMaximized);
      });
    };

    initWindow();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleMinimize = async () => {
    if (!appWindow) return;
    try {
      await appWindow.minimize();
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  };

  const handleMaximize = async () => {
    if (!appWindow) return;
    try {
      await appWindow.toggleMaximize();
    } catch (error) {
      console.error('Failed to toggle maximize:', error);
    }
  };

  const handleClose = async () => {
    if (!appWindow) return;
    try {
      await appWindow.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  };

  const openDebug = () => {
    console.log('Open Debug Console');
    // Future implementation: WebviewWindow for debug
  };

  const checkUpdates = () => {
    console.log('Checking for updates...');
  };

  const showCredits = () => {
    console.log('Showing credits...');
  };

  const reportIssue = () => {
    window.open('https://github.com/twmprogramer/MC-Server-Wrapper/issues', '_blank');
  };

  return (
    <div className="titlebar">
      <div className="titlebar-drag-region">
        <div className="titlebar-drag-handle" />
      </div>

      <div className="titlebar-left">
        <div className="titlebar-logo-container">
          <img src="/app-icon.png" alt="Logo" className="titlebar-logo" />
        </div>
        <div className="titlebar-title-container">
          <div className="flex items-center gap-1">
            <span className="titlebar-title">Minecraft Server Wrapper</span>
            <span className="titlebar-tag">v{appVersion}</span>
          </div>
          <span className="titlebar-subtitle">Powered by twmprogramer</span>
        </div>
      </div>

      <div className="titlebar-right">
        <div className="titlebar-actions">
          <button
            className="titlebar-action-button"
            title="Debug Console"
            onClick={openDebug}
          >
            <Bug size={14} />
          </button>
          <button
            className="titlebar-action-button"
            title="Check for Updates"
            onClick={checkUpdates}
          >
            <RefreshCw size={14} />
          </button>
          <button
            className="titlebar-action-button"
            title="Credits"
            onClick={showCredits}
          >
            <Users size={14} />
          </button>
          <button
            className="titlebar-action-button"
            title="Report Issue"
            onClick={reportIssue}
          >
            <Github size={14} />
          </button>
        </div>

        <div className="titlebar-controls">
          <div
            className="titlebar-control-button"
            onClick={handleMinimize}
            title="Minimize"
          >
            <Minus size={14} />
          </div>
          <div
            className="titlebar-control-button"
            onClick={handleMaximize}
            title={isMaximized ? "Restore" : "Maximize"}
          >
            {isMaximized ? <Copy size={12} /> : <Square size={12} />}
          </div>
          <div
            className="titlebar-control-button close"
            onClick={handleClose}
            title="Close"
          >
            <X size={14} />
          </div>
        </div>
      </div>
    </div>
  );
}
