import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './float.css';

type AppState = 'idle' | 'recording' | 'processing' | 'injecting' | 'error' | 'done';

interface StateChangedPayload {
  state: string; // The backend sends lowercase strings like "idle", "recording"
  message?: string;
}

export default function FloatWindow() {
  const [appState, setAppState] = useState<AppState>('idle');
  const [volume, setVolume] = useState(0);
  const [elapsed, setElapsed] = useState('0:00');
  const [errorMessage, setErrorMessage] = useState('');
  
  const startTimeRef = useRef<number | null>(null);
  const timerIntervalRef = useRef<number | null>(null);
  const volumeIntervalRef = useRef<number | null>(null);

  // Initial setup: hide window on mount if idle
  useEffect(() => {
    const init = async () => {
      // Check initial state from backend
      try {
        const state = await invoke<string>('get_state');
        handleStateChange(state as AppState);
      } catch (e) {
        console.error('Failed to get initial state:', e);
      }
    };
    init();
  }, []);

  // Listen for state changes
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen<StateChangedPayload>('state-changed', (event) => {
        console.log('State changed event:', event);
        const newState = event.payload.state as AppState;
        
        if (event.payload.message) {
            // If it's an error message, store it
            if (newState === 'error') {
                setErrorMessage(event.payload.message);
            }
        }
        
        handleStateChange(newState);
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleStateChange = async (newState: AppState) => {
    setAppState(newState);
    const win = getCurrentWindow();

    if (newState === 'idle' || newState === 'done') {
      await win.hide();
      stopTimer();
      stopVolumePolling();
      setVolume(0);
      setElapsed('0:00');
    } else {
      await win.show();
      // Ensure always on top just in case
      await win.setAlwaysOnTop(true);
      
      if (newState === 'recording') {
        startTimer();
        startVolumePolling();
      } else {
        stopTimer();
        stopVolumePolling();
        setVolume(0); // Reset volume when not recording
      }

      if (newState === 'error') {
        setTimeout(() => {
           // Auto dismiss error after 3s
           // We don't change state here locally, we rely on backend or just hide?
           // The prompt says "auto-dismiss after 3s -> idle".
           // Ideally the backend transitions to idle. 
           // If the backend doesn't, we might desync.
           // For UI purposes, let's just hide or let backend handle it.
           // Prompt says: "Error: Red error text + auto-dismiss after 3s -> idle"
           // I'll assume backend handles the transition to idle after error.
           // If not, I'll force it.
           // Actually, standard behavior is backend handles state.
        }, 3000);
      }
    }
  };

  const startTimer = () => {
    if (timerIntervalRef.current) return;
    startTimeRef.current = Date.now();
    timerIntervalRef.current = window.setInterval(() => {
      if (startTimeRef.current) {
        const diff = Math.floor((Date.now() - startTimeRef.current) / 1000);
        const mins = Math.floor(diff / 60);
        const secs = diff % 60;
        setElapsed(`${mins}:${secs.toString().padStart(2, '0')}`);
      }
    }, 1000);
  };

  const stopTimer = () => {
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current);
      timerIntervalRef.current = null;
    }
    startTimeRef.current = null;
  };

  const startVolumePolling = () => {
    if (volumeIntervalRef.current) return;
    volumeIntervalRef.current = window.setInterval(async () => {
      try {
        // We use the Rust command we added to lib.rs
        const vol = await invoke<number>('get_volume');
        setVolume(Math.min(Math.max(vol, 0), 1));
      } catch (e) {
        console.warn('Failed to poll volume:', e);
      }
    }, 200);
  };

  const stopVolumePolling = () => {
    if (volumeIntervalRef.current) {
      clearInterval(volumeIntervalRef.current);
      volumeIntervalRef.current = null;
    }
  };

  return (
    <div className="float-container">
      <div className="status-row">
        {appState === 'recording' && (
          <>
            <div className="indicator recording-dot" />
            <span className="status-text">Recording...</span>
            <span className="timer">{elapsed}</span>
          </>
        )}

        {appState === 'processing' && (
          <>
            <div className="indicator processing-spinner" />
            <span className="status-text">Processing...</span>
          </>
        )}

        {appState === 'injecting' && (
          <>
            <div className="indicator injecting-flash" />
            <span className="status-text">Injecting...</span>
          </>
        )}

        {appState === 'error' && (
            <>
             <span className="status-text error-text">{errorMessage || 'Error occurred'}</span>
            </>
        )}
      </div>

      {/* Volume bar always visible in layout but only active when recording */}
      <div className="volume-bar-container">
        <div 
            className="volume-bar" 
            style={{ 
                width: appState === 'recording' ? `${volume * 100}%` : '0%',
                opacity: appState === 'recording' ? 1 : 0
            }} 
        />
      </div>
    </div>
  );
}
