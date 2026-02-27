import React from 'react';
import ReactDOM from 'react-dom/client';
import FloatWindow from './FloatWindow.tsx';
import './float.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <FloatWindow />
  </React.StrictMode>
);
