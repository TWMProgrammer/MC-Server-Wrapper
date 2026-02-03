import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { ToastProvider } from './hooks/useToast'
import { AppSettingsProvider } from './hooks/useAppSettings'
import ErrorBoundary from './components/ErrorBoundary'
import './index.css'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <AppSettingsProvider>
        <ToastProvider>
          <App />
        </ToastProvider>
      </AppSettingsProvider>
    </ErrorBoundary>
  </React.StrictMode>,
)
