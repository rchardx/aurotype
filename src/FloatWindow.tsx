import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './float.css';

type AppState = 'idle' | 'recording' | 'processing' | 'injecting' | 'error' | 'done' | 'copy_available';

interface StateChangedPayload {
  state: string;
  message?: string;
}

const CheckIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className="injecting-icon">
    <polyline points="20 6 9 17 4 12" />
  </svg>
);

const ErrorIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="error-icon">
    <circle cx="12" cy="12" r="10" />
    <line x1="15" y1="9" x2="9" y2="15" />
    <line x1="9" y1="9" x2="15" y2="15" />
  </svg>
);

const ClipboardIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="clipboard-icon">
    <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
    <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
  </svg>
);

export default function FloatWindow() {
  const [appState, setAppState] = useState<AppState>('idle');
  const [volume, setVolume] = useState(0);
  const [elapsed, setElapsed] = useState('0:00');
  const [errorMessage, setErrorMessage] = useState('');
  const [copyText, setCopyText] = useState('');
  
  const startTimeRef = useRef<number | null>(null);
  const timerIntervalRef = useRef<number | null>(null);
  const volumeIntervalRef = useRef<number | null>(null);
  const copyDismissRef = useRef<number | null>(null);

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

        const newState = event.payload.state as AppState;
        
        if (newState === 'copy_available' && event.payload.message) {
          setCopyText(event.payload.message);
        }

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
      // Clear copy text on idle/done
      if (newState === 'idle') setCopyText('');
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

      // Special handling for copy_available - ensure window is visible and sized correctly
      if (newState === 'copy_available') {
        copyDismissRef.current = window.setTimeout(async () => {
          copyDismissRef.current = null;
          await invoke('cancel');
        }, 2000);
      } else if (copyDismissRef.current !== null) {
        clearTimeout(copyDismissRef.current);
        copyDismissRef.current = null;
      }

      if (newState === 'error') {
        setTimeout(async () => {
          const w = getCurrentWindow();
          await w.hide();
          setAppState('idle');
          setErrorMessage('');
          setVolume(0);
          setElapsed('0:00');
          stopTimer();
          stopVolumePolling();
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
    }, 100); // Polling faster for smoother UI
  };

  const stopVolumePolling = () => {
    if (volumeIntervalRef.current) {
      clearInterval(volumeIntervalRef.current);
      volumeIntervalRef.current = null;
    }
  };

  const handleCopy = async () => {
    if (copyDismissRef.current !== null) {
      clearTimeout(copyDismissRef.current);
      copyDismissRef.current = null;
    }
    await invoke('copy_to_clipboard', { text: copyText });
    await invoke('cancel');
  };

  return (
    <div className={`float-wrapper ${appState}`}>
      {/* The main bubble */}
      <div className="bubble-container">
        {/* Pulse animation ring (only when recording) */}
        {appState === 'recording' && <div className="pulse-ring" />}


        {/* Central Icon */}
        <div className="icon-center">
          {appState === 'recording' ? (
            <div className="waveform-bars">
              {[0.4, 0.6, 0.8, 1.0, 0.8, 0.6, 0.4].map((multiplier, i) => (
                <div 
                  key={i} 
                  className="wave-bar" 
                  style={{ 
                    height: `${Math.max(8, volume * 120 * multiplier)}px`,
                    animationDelay: `${i * 0.1}s`
                  }} 
                />
              ))}
            </div>
          ) : null}
          {appState === 'processing' && <div className="spinner" />}
          {appState === 'injecting' && <CheckIcon />}
          {appState === 'error' && <ErrorIcon />}
          {appState === 'copy_available' && <ClipboardIcon />}
        </div>
      </div>

      {/* Info text beside the bubble */}
      <div className="info-panel">
        {appState === 'recording' && (
          <>
            <div className="status-label">Recording</div>
            <div className="timer-text">{elapsed}</div>
          </>
        )}
        {appState === 'processing' && (
           <div className="status-label transcribing">Transcribing...</div>
        )}
        {appState === 'injecting' && (
           <div className="status-label success">Complete</div>
        )}
        {appState === 'error' && (
           <div className="status-label error">{errorMessage || 'Error'}</div>
        )}
        {appState === 'copy_available' && (
          <div className="copy-panel">
            <div className="copy-text-preview" title={copyText}>
              {copyText.length > 30 ? copyText.substring(0, 30) + '...' : copyText}
            </div>
            <button className="copy-btn" onClick={handleCopy}>
              Copy
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
