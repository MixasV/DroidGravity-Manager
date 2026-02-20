import { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { Plus, Database, Globe, FileClock, Loader2, CheckCircle2, XCircle, Copy, Check } from 'lucide-react';
import { useAccountStore } from '../../stores/useAccountStore';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { request as invoke } from '../../utils/request';

interface AddAccountDialogProps {
    onAdd: (email: string, refreshToken: string) => Promise<void>;
}

type Status = 'idle' | 'loading' | 'success' | 'error';

function AddAccountDialog({ onAdd }: AddAccountDialogProps) {
    const { t } = useTranslation();
    const [isOpen, setIsOpen] = useState(false);
    const [provider, setProvider] = useState<'gemini' | 'kiro'>('kiro'); // Allow provider selection
    const [activeTab, setActiveTab] = useState<'oauth' | 'token' | 'import' | 'manual'>('oauth');
    const [refreshToken, setRefreshToken] = useState('');
    const [oauthUrl, setOauthUrl] = useState('');
    const [oauthUrlCopied, setOauthUrlCopied] = useState(false);
    const [manualCode, setManualCode] = useState(''); // For manual Kiro OAuth code input
    
    // Manual token input fields for Kiro
    const [manualAccessToken, setManualAccessToken] = useState('');
    const [manualRefreshToken, setManualRefreshToken] = useState('');

    // UI State
    const [status, setStatus] = useState<Status>('idle');
    const [message, setMessage] = useState('');

    const { completeOAuthLogin, cancelOAuthLogin, importFromDb, importV1Accounts, importFromCustomDb, fetchAccounts } = useAccountStore();

    const oauthUrlRef = useRef(oauthUrl);
    const statusRef = useRef(status);
    const activeTabRef = useRef(activeTab);
    const isOpenRef = useRef(isOpen);

    useEffect(() => {
        oauthUrlRef.current = oauthUrl;
        statusRef.current = status;
        activeTabRef.current = activeTab;
        isOpenRef.current = isOpen;
    }, [oauthUrl, status, activeTab, isOpen]);

    // Reset state when dialog opens or tab changes
    useEffect(() => {
        if (isOpen) {
            resetState();
        }
    }, [isOpen, activeTab]);

    // Change default tab when provider changes
    useEffect(() => {
        if (provider === 'kiro') {
            setActiveTab('manual');
        } else {
            setActiveTab('oauth');
        }
    }, [provider]);

    // Listen for OAuth URL
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen('oauth-url-generated', (event) => {
                setOauthUrl(event.payload as string);
                // 自动复制到剪贴板? 可选，这里只设置状态让用户手动复制
            });
        };

        setupListener();

        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    // Listen for OAuth callback completion (user may open the URL manually without clicking Start)
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen('oauth-callback-received', async () => {
                if (!isOpenRef.current) return;
                if (activeTabRef.current !== 'oauth') return;
                if (statusRef.current === 'loading' || statusRef.current === 'success') return;
                if (!oauthUrlRef.current) return;

                // Auto-complete: exchange code and save account (no browser open)
                setStatus('loading');
                setMessage(`${t('accounts.add.tabs.oauth')}...`);

                try {
                    await completeOAuthLogin();
                    setStatus('success');
                    setMessage(`${t('accounts.add.tabs.oauth')} ${t('common.success')}!`);
                    setTimeout(() => {
                        setIsOpen(false);
                        resetState();
                    }, 1500);
                } catch (error) {
                    setStatus('error');
                    let errorMsg = String(error);
                    if (errorMsg.includes('Refresh Token') || errorMsg.includes('refresh_token')) {
                        setMessage(errorMsg);
                    } else if (errorMsg.includes('Tauri') || errorMsg.toLowerCase().includes('environment') || errorMsg.includes('环境')) {
                        setMessage(t('common.environment_error', { error: errorMsg }));
                    } else {
                        setMessage(`${t('accounts.add.tabs.oauth')} ${t('common.error')}: ${errorMsg}`);
                    }
                }
            });
        };

        setupListener();

        return () => {
            if (unlisten) unlisten();
        };
    }, [completeOAuthLogin, t]);

    // Pre-generate OAuth URL when dialog opens on OAuth tab OR when provider changes to Kiro
    useEffect(() => {
        if (!isOpen) return;
        if (activeTab !== 'oauth') return;
        if (oauthUrl && provider === 'kiro') return; // Don't regenerate if we already have Kiro URL

        const prepareCommand = provider === 'kiro' ? 'prepare_kiro_oauth_url' : 'prepare_oauth_url';
        
        invoke<string>(prepareCommand)
            .then((url) => {
                // Set directly (also emitted via event), to avoid any race if event is missed.
                if (typeof url === 'string' && url.length > 0) setOauthUrl(url);
            })
            .catch((e) => {
                console.error('Failed to prepare OAuth URL:', e);
            });
    }, [isOpen, activeTab, oauthUrl, provider]);

    // If user navigates away from OAuth tab, cancel prepared flow to release the port.
    useEffect(() => {
        if (!isOpen) return;
        if (activeTab === 'oauth') return;
        if (!oauthUrl) return;

        cancelOAuthLogin().catch(() => { });
        setOauthUrl('');
        setOauthUrlCopied(false);
    }, [isOpen, activeTab]);

    const resetState = () => {
        setStatus('idle');
        setMessage('');
        setRefreshToken('');
        setOauthUrl('');
        setOauthUrlCopied(false);
        setManualCode('');
        setManualAccessToken('');
        setManualRefreshToken('');
    };

    const handleAction = async (
        actionName: string,
        actionFn: () => Promise<any>,
        options?: { clearOauthUrl?: boolean }
    ) => {
        setStatus('loading');
        setMessage(`${actionName}...`);
        if (options?.clearOauthUrl !== false) {
            setOauthUrl(''); // Clear previous URL
        }
        try {
            await actionFn();
            setStatus('success');
            setMessage(`${actionName} ${t('common.success')}!`);

            // 延迟关闭,让用户看到成功状态
            setTimeout(() => {
                setIsOpen(false);
                resetState();
            }, 1500);
        } catch (error) {
            setStatus('error');

            // 改进错误信息显示
            let errorMsg = String(error);

            // 如果是 refresh_token 缺失错误,显示完整信息(包含解决方案)
            if (errorMsg.includes('Refresh Token') || errorMsg.includes('refresh_token')) {
                setMessage(errorMsg);
            } else if (errorMsg.includes('Tauri') || errorMsg.toLowerCase().includes('environment') || errorMsg.includes('环境')) {
                // 环境错误
                setMessage(t('common.environment_error', { error: errorMsg }));
            } else {
                // 其他错误
                setMessage(`${actionName} ${t('common.error')}: ${errorMsg}`);
            }
        }
    };

    const handleSubmit = async () => {
        if (!refreshToken) {
            setStatus('error');
            setMessage(t('accounts.add.token.error_token'));
            return;
        }

        setStatus('loading');

        // 1. 尝试解析输入
        let tokens: string[] = [];
        const input = refreshToken.trim();

        try {
            // 尝试解析为 JSON
            if (input.startsWith('[') && input.endsWith(']')) {
                const parsed = JSON.parse(input);
                if (Array.isArray(parsed)) {
                    tokens = parsed
                        .map((item: any) => item.refresh_token)
                        .filter((t: any) => typeof t === 'string' && t.startsWith('1//'));
                }
            }
        } catch (e) {
            // JSON 解析失败,忽略
            console.debug('JSON parse failed, falling back to regex', e);
        }

        // 2. 如果 JSON 解析没有结果,尝试正则提取 (或者输入不是 JSON)
        if (tokens.length === 0) {
            const regex = /1\/\/[a-zA-Z0-9_\-]+/g;
            const matches = input.match(regex);
            if (matches) {
                tokens = matches;
            }
        }

        // 去重
        tokens = [...new Set(tokens)];

        if (tokens.length === 0) {
            setStatus('error');
            setMessage(t('accounts.add.token.error_token')); // 或者提示"未找到有效 Token"
            return;
        }

        // 3. 批量添加
        let successCount = 0;
        let failCount = 0;

        for (let i = 0; i < tokens.length; i++) {
            const currentToken = tokens[i];
            setMessage(t('accounts.add.token.batch_progress', { current: i + 1, total: tokens.length }));

            try {
                await onAdd("", currentToken);
                successCount++;
            } catch (error) {
                console.error(`Failed to add token ${i + 1}:`, error);
                failCount++;
            }
            // 稍微延迟一下,避免太快
            await new Promise(r => setTimeout(r, 100));
        }

        // 4. 结果反馈
        if (successCount === tokens.length) {
            setStatus('success');
            setMessage(t('accounts.add.token.batch_success', { count: successCount }));
            setTimeout(() => {
                setIsOpen(false);
                resetState();
            }, 1500);
        } else if (successCount > 0) {
            // 部分成功
            setStatus('success'); // 还是用绿色,但提示部分失败
            setMessage(t('accounts.add.token.batch_partial', { success: successCount, fail: failCount }));
            // 不自动关闭,让用户看到结果
        } else {
            // 全部失败
            setStatus('error');
            setMessage(t('accounts.add.token.batch_fail'));
        }
    };

    const handleOAuth = () => {
        // Default flow: opens the default browser and completes automatically.
        // (If user opened the URL manually, completion is also triggered by oauth-callback-received.)
        const command = provider === 'kiro' ? 'start_kiro_oauth_login' : 'start_oauth_login';
        handleAction(t('accounts.add.tabs.oauth'), () => invoke(command), { clearOauthUrl: false });
    };

    const handleCompleteOAuth = () => {
        // Manual flow: user already authorized in their preferred browser, just finish the flow.
        const command = provider === 'kiro' ? 'complete_kiro_oauth_login' : 'complete_oauth_login';
        handleAction(t('accounts.add.tabs.oauth'), async () => {
            await invoke(command);
            // Refresh accounts list to show the new account
            await fetchAccounts();
        }, { clearOauthUrl: false });
    };

    const handleSubmitManualTokens = async () => {
        if (!manualAccessToken.trim() || !manualRefreshToken.trim()) {
            setStatus('error');
            setMessage('Please enter both Access Token and Refresh Token');
            return;
        }

        handleAction('Add Kiro Account with Manual Tokens', async () => {
            // Use manual_token_input command for Kiro
            await invoke('manual_kiro_token_input', {
                accessToken: manualAccessToken.trim(),
                refreshToken: manualRefreshToken.trim(),
                expiresIn: 3600 // Default 1 hour
            });
            
            // Refresh accounts list to show the new Kiro account
            await fetchAccounts();
        });
    };

    const handleSubmitManualCode = () => {
        if (!manualCode.trim()) {
            setStatus('error');
            setMessage('Please enter the authorization code or callback URL');
            return;
        }

        // Extract code from URL if user pasted full callback URL
        let code = manualCode.trim();
        try {
            if (code.includes('code=')) {
                const url = new URL(code.startsWith('http') ? code : `http://localhost:3128${code}`);
                const codeParam = url.searchParams.get('code');
                if (codeParam) {
                    code = codeParam;
                }
            }
        } catch (e) {
            // If URL parsing fails, assume it's already just the code
        }

        handleAction('Submit Authorization Code', async () => {
            await invoke('submit_kiro_oauth_code', { code });
            // Refresh accounts list to show the new Kiro account
            await fetchAccounts();
        }, { clearOauthUrl: false });
    };

    const handleCopyUrl = async () => {
        if (oauthUrl) {
            try {
                await navigator.clipboard.writeText(oauthUrl);
                setOauthUrlCopied(true);
                window.setTimeout(() => setOauthUrlCopied(false), 1500);
            } catch (err) {
                console.error('Failed to copy: ', err);
            }
        }
    };

    const handleImportDb = () => {
        handleAction(t('accounts.add.tabs.import'), importFromDb);
    };

    const handleImportV1 = () => {
        handleAction(t('accounts.add.import.btn_v1'), importV1Accounts);
    };

    const handleImportCustomDb = async () => {
        try {
            const selected = await open({
                multiple: false,
                filters: [{
                    name: 'VSCode DB',
                    extensions: ['vscdb']
                }, {
                    name: 'All Files',
                    extensions: ['*']
                }]
            });

            if (selected && typeof selected === 'string') {
                handleAction(t('accounts.add.import.btn_custom_db') || 'Import Custom DB', () => importFromCustomDb(selected));
            }
        } catch (err) {
            console.error('Failed to open dialog:', err);
        }
    };

    // 状态提示组件
    const StatusAlert = () => {
        if (status === 'idle' || !message) return null;

        const styles = {
            loading: 'alert-info',
            success: 'alert-success',
            error: 'alert-error'
        };

        const icons = {
            loading: <Loader2 className="w-5 h-5 animate-spin" />,
            success: <CheckCircle2 className="w-5 h-5" />,
            error: <XCircle className="w-5 h-5" />
        };

        return (
            <div className={`alert ${styles[status]} mb-4 text-sm py-2 shadow-sm`}>
                {icons[status]}
                <span>{message}</span>
            </div>
        );
    };

    return (
        <>
            <button
                className="px-4 py-2 bg-white dark:bg-base-100 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors flex items-center gap-2 shadow-sm border border-gray-200/50 dark:border-base-300"
                onClick={() => setIsOpen(true)}
            >
                <Plus className="w-4 h-4" />
                {t('accounts.add_account')}
            </button>

            {isOpen && createPortal(
                <div className="modal modal-open z-[100]">
                    {/* Draggable Top Region */}
                    <div data-tauri-drag-region className="fixed top-0 left-0 right-0 h-8 z-[110]" />

                    <div className="modal-box bg-white dark:bg-base-100 text-gray-900 dark:text-base-content">
                        <h3 className="font-bold text-lg mb-4">{t('accounts.add.title')}</h3>

                        {/* Provider Selection */}
                        <div className="mb-4">
                            <label className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 block">
                                Select Provider:
                            </label>
                            <div className="grid grid-cols-2 gap-2">
                                <button
                                    className={`p-3 rounded-xl border transition-all ${
                                        provider === 'gemini'
                                            ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800 text-blue-700 dark:text-blue-300'
                                            : 'bg-gray-50 dark:bg-base-200 border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-base-300'
                                    }`}
                                    onClick={() => setProvider('gemini')}
                                >
                                    <div className="flex items-center gap-2">
                                        <div className="w-6 h-6 bg-blue-600 rounded-lg flex items-center justify-center">
                                            <span className="text-white text-xs font-bold">G</span>
                                        </div>
                                        <div className="text-left">
                                            <div className="font-medium text-sm">Google Gemini</div>
                                            <div className="text-xs opacity-70">OAuth via Google</div>
                                        </div>
                                    </div>
                                </button>
                                <button
                                    className={`p-3 rounded-xl border transition-all ${
                                        provider === 'kiro'
                                            ? 'bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800 text-purple-700 dark:text-purple-300'
                                            : 'bg-gray-50 dark:bg-base-200 border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-base-300'
                                    }`}
                                    onClick={() => setProvider('kiro')}
                                >
                                    <div className="flex items-center gap-2">
                                        <div className="w-6 h-6 bg-purple-600 rounded-lg flex items-center justify-center">
                                            <span className="text-white text-xs font-bold">🚀</span>
                                        </div>
                                        <div className="text-left">
                                            <div className="font-medium text-sm">Kiro</div>
                                            <div className="text-xs opacity-70">OAuth via Google</div>
                                        </div>
                                    </div>
                                </button>
                            </div>
                        </div>

                        {/* Tab 导航 -胶囊风格 */}
                        <div className={`bg-gray-100 dark:bg-base-200 p-1 rounded-xl mb-6 grid gap-1 ${provider === 'kiro' ? 'grid-cols-2' : 'grid-cols-3'}`}>
                            {provider === 'gemini' && (
                                <>
                                    <button
                                        className={`py-2 px-3 rounded-lg text-sm font-medium transition-all duration-200 ${activeTab === 'oauth'
                                            ? 'bg-white dark:bg-base-100 shadow-sm text-blue-600 dark:text-blue-400'
                                            : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-200/50 dark:hover:bg-base-300'
                                            } `}
                                        onClick={() => setActiveTab('oauth')}
                                    >
                                        {t('accounts.add.tabs.oauth')}
                                    </button>
                                    <button
                                        className={`py-2 px-3 rounded-lg text-sm font-medium transition-all duration-200 ${activeTab === 'token'
                                            ? 'bg-white dark:bg-base-100 shadow-sm text-blue-600 dark:text-blue-400'
                                            : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-200/50 dark:hover:bg-base-300'
                                            } `}
                                        onClick={() => setActiveTab('token')}
                                    >
                                        {t('accounts.add.tabs.token')}
                                    </button>
                                </>
                            )}
                            {provider === 'kiro' && (
                                <button
                                    className={`py-2 px-3 rounded-lg text-sm font-medium transition-all duration-200 ${activeTab === 'manual'
                                        ? 'bg-white dark:bg-base-100 shadow-sm text-blue-600 dark:text-blue-400'
                                        : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-200/50 dark:hover:bg-base-300'
                                        } `}
                                    onClick={() => setActiveTab('manual')}
                                >
                                    Manual
                                </button>
                            )}
                            <button
                                className={`py-2 px-3 rounded-lg text-sm font-medium transition-all duration-200 ${activeTab === 'import'
                                    ? 'bg-white dark:bg-base-100 shadow-sm text-blue-600 dark:text-blue-400'
                                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-200/50 dark:hover:bg-base-300'
                                    } `}
                                onClick={() => setActiveTab('import')}
                            >
                                {t('accounts.add.tabs.import')}
                            </button>
                        </div>

                        {/* 状态提示区 */}
                        <StatusAlert />

                        <div className="min-h-[200px]">
                            {/* OAuth 授权 */}
                            {activeTab === 'oauth' && (
                                <div className="space-y-6 py-4">
                                    <div className="text-center space-y-3">
                                        <div className="bg-blue-50 dark:bg-blue-900/20 p-6 rounded-full w-20 h-20 mx-auto flex items-center justify-center">
                                            <Globe className="w-10 h-10 text-blue-500" />
                                        </div>
                                        <div className="space-y-1">
                                            <h4 className="font-medium text-gray-900 dark:text-gray-100">{t('accounts.add.oauth.recommend')}</h4>
                                            <p className="text-sm text-gray-500 dark:text-gray-400 max-w-xs mx-auto">
                                                {t('accounts.add.oauth.desc')}
                                            </p>
                                        </div>
                                    </div>
                                    <div className="space-y-3">
                                        <button
                                            className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-xl shadow-lg shadow-blue-500/20 transition-all flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed"
                                            onClick={handleOAuth}
                                            disabled={status === 'loading' || status === 'success'}
                                        >
                                            {status === 'loading' ? t('accounts.add.oauth.btn_waiting') : t('accounts.add.oauth.btn_start')}
                                        </button>

                                        {oauthUrl && (
                                            <div className="space-y-2">
                                                <div className="text-[11px] text-gray-500 dark:text-gray-400 text-left">
                                                    {t('accounts.add.oauth.link_label')}
                                                </div>
                                                <button
                                                    type="button"
                                                    className="w-full px-4 py-2 bg-white dark:bg-base-100 text-gray-600 dark:text-gray-300 text-sm font-medium rounded-xl border border-dashed border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-base-200 transition-all flex items-center gap-2"
                                                    onClick={handleCopyUrl}
                                                    title={t('accounts.add.oauth.link_click_to_copy')}
                                                >
                                                    {oauthUrlCopied ? (
                                                        <Check className="w-3.5 h-3.5 text-emerald-600" />
                                                    ) : (
                                                        <Copy className="w-3.5 h-3.5" />
                                                    )}
                                                    <code className="text-[11px] font-mono truncate flex-1 text-left">
                                                        {oauthUrl}
                                                    </code>
                                                    <span className="text-[11px] whitespace-nowrap">
                                                        {oauthUrlCopied ? t('accounts.add.oauth.copied') : t('accounts.add.oauth.copy_link')}
                                                    </span>
                                                </button>

                                                <button
                                                    type="button"
                                                    className="w-full px-4 py-2 bg-white dark:bg-base-100 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 transition-all flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed"
                                                    onClick={handleCompleteOAuth}
                                                    disabled={status === 'loading' || status === 'success'}
                                                >
                                                    <CheckCircle2 className="w-4 h-4" />
                                                    {t('accounts.add.oauth.btn_finish')}
                                                </button>

                                                {provider === 'kiro' && (
                                                    <div className="space-y-2 pt-2 border-t border-gray-200 dark:border-gray-600">
                                                        <div className="text-[11px] text-gray-500 dark:text-gray-400 text-left">
                                                            Or paste authorization code/callback URL manually:
                                                        </div>
                                                        <div className="flex gap-2">
                                                            <input
                                                                type="text"
                                                                className="flex-1 px-3 py-2 text-xs font-mono bg-white dark:bg-base-100 border border-gray-300 dark:border-base-300 rounded-lg focus:outline-none focus:border-blue-500 transition-colors"
                                                                placeholder="Paste code or full callback URL here..."
                                                                value={manualCode}
                                                                onChange={(e) => setManualCode(e.target.value)}
                                                                disabled={status === 'loading' || status === 'success'}
                                                            />
                                                            <button
                                                                type="button"
                                                                className="px-3 py-2 bg-green-600 hover:bg-green-700 text-white text-xs font-medium rounded-lg transition-all disabled:opacity-70 disabled:cursor-not-allowed"
                                                                onClick={handleSubmitManualCode}
                                                                disabled={status === 'loading' || status === 'success' || !manualCode.trim()}
                                                            >
                                                                Submit
                                                            </button>
                                                        </div>
                                                        <p className="text-[10px] text-gray-400">
                                                            You can authorize on another device and paste the code or full callback URL here.
                                                        </p>
                                                    </div>
                                                )}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            )}

                            {/* Manual Token Input for Kiro */}
                            {activeTab === 'manual' && provider === 'kiro' && (
                                <div className="space-y-4 py-2">
                                    <div className="bg-yellow-50 dark:bg-yellow-900/20 p-4 rounded-lg border border-yellow-200 dark:border-yellow-800">
                                        <h4 className="font-medium text-yellow-800 dark:text-yellow-200 mb-2">
                                            📋 Manual Token Input from Browser Cookies
                                        </h4>
                                        <p className="text-sm text-yellow-700 dark:text-yellow-300 mb-3">
                                            Extract tokens from browser cookies after authorization:
                                        </p>
                                        <ol className="text-xs text-yellow-600 dark:text-yellow-400 space-y-1 ml-4 list-decimal">
                                            <li>Visit <a href="https://app.kiro.dev/" target="_blank" rel="noopener noreferrer" className="underline hover:text-yellow-800 dark:hover:text-yellow-200">https://app.kiro.dev/</a> and sign in with <strong>Google</strong> (only Google auth is currently supported)</li>
                                            <li>After successful login, open DevTools (F12) → Application → Cookies → app.kiro.dev</li>
                                            <li>Find and copy <code className="bg-yellow-200 dark:bg-yellow-800 px-1 rounded">AccessToken</code> value</li>
                                            <li>Find and copy <code className="bg-yellow-200 dark:bg-yellow-800 px-1 rounded">RefreshToken</code> value</li>
                                            <li>Paste both tokens below and click Add Account</li>
                                        </ol>
                                    </div>
                                    
                                    <div className="space-y-3">
                                        <div>
                                            <label className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 block">
                                                Access Token:
                                            </label>
                                            <textarea
                                                className="w-full h-20 px-3 py-2 text-xs font-mono bg-white dark:bg-base-100 border border-gray-300 dark:border-base-300 rounded-lg focus:outline-none focus:border-blue-500 transition-colors resize-none"
                                                placeholder="Paste AccessToken from browser cookies here..."
                                                value={manualAccessToken}
                                                onChange={(e) => setManualAccessToken(e.target.value)}
                                                disabled={status === 'loading' || status === 'success'}
                                            />
                                        </div>
                                        
                                        <div>
                                            <label className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 block">
                                                Refresh Token:
                                            </label>
                                            <textarea
                                                className="w-full h-20 px-3 py-2 text-xs font-mono bg-white dark:bg-base-100 border border-gray-300 dark:border-base-300 rounded-lg focus:outline-none focus:border-blue-500 transition-colors resize-none"
                                                placeholder="Paste RefreshToken from browser cookies here..."
                                                value={manualRefreshToken}
                                                onChange={(e) => setManualRefreshToken(e.target.value)}
                                                disabled={status === 'loading' || status === 'success'}
                                            />
                                        </div>
                                    </div>
                                </div>
                            )}

                            {/* Refresh Token */}
                            {activeTab === 'token' && (
                                <div className="space-y-4 py-2">
                                    <div className="bg-gray-50 dark:bg-base-200 p-4 rounded-lg border border-gray-200 dark:border-base-300">
                                        <div className="flex justify-between items-center mb-2">
                                            <span className="text-sm font-medium text-gray-500 dark:text-gray-400">{t('accounts.add.token.label')}</span>
                                        </div>
                                        <textarea
                                            className="textarea textarea-bordered w-full h-32 font-mono text-xs leading-relaxed focus:outline-none focus:border-blue-500 transition-colors bg-white dark:bg-base-100 text-gray-900 dark:text-base-content border-gray-300 dark:border-base-300 placeholder:text-gray-400"
                                            placeholder={t('accounts.add.token.placeholder')}
                                            value={refreshToken}
                                            onChange={(e) => setRefreshToken(e.target.value)}
                                            disabled={status === 'loading' || status === 'success'}
                                        />
                                        <p className="text-[10px] text-gray-400 mt-2">
                                            {t('accounts.add.token.hint')}
                                        </p>
                                    </div>
                                </div>
                            )}

                            {/* 从数据库导入 */}
                            {activeTab === 'import' && (
                                <div className="space-y-6 py-2">
                                    <div className="space-y-2">
                                        <h4 className="font-semibold flex items-center gap-2 text-gray-800 dark:text-gray-200">
                                            <Database className="w-4 h-4 text-gray-600 dark:text-gray-400" />
                                            {t('accounts.add.import.scheme_a')}
                                        </h4>
                                        <p className="text-xs text-gray-500 dark:text-gray-400">
                                            {t('accounts.add.import.scheme_a_desc')}
                                        </p>
                                        <button
                                            className="w-full px-4 py-3 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-blue-50 dark:hover:bg-blue-900/20 hover:border-blue-200 dark:hover:border-blue-800 hover:text-blue-600 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed mb-2 shadow-sm"
                                            onClick={handleImportDb}
                                            disabled={status === 'loading' || status === 'success'}
                                        >
                                            <CheckCircle2 className="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity" />
                                            {t('accounts.add.import.btn_db')}
                                        </button>
                                        <button
                                            className="w-full px-4 py-3 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 hover:border-indigo-200 dark:hover:border-indigo-800 hover:text-indigo-600 dark:hover:text-indigo-400 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
                                            onClick={handleImportCustomDb}
                                            disabled={status === 'loading' || status === 'success'}
                                        >
                                            <Database className="w-4 h-4" />
                                            {t('accounts.add.import.btn_custom_db') || 'Custom DB (state.vscdb)'}
                                        </button>
                                    </div>

                                    <div className="divider text-xs text-gray-300 dark:text-gray-600">{t('accounts.add.import.or')}</div>

                                    <div className="space-y-2">
                                        <h4 className="font-semibold flex items-center gap-2 text-gray-800 dark:text-gray-200">
                                            <FileClock className="w-4 h-4 text-gray-600 dark:text-gray-400" />
                                            {t('accounts.add.import.scheme_b')}
                                        </h4>
                                        <p className="text-xs text-gray-500 dark:text-gray-400">
                                            {t('accounts.add.import.scheme_b_desc')}
                                        </p>
                                        <button
                                            className="w-full px-4 py-3 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-emerald-50 dark:hover:bg-emerald-900/20 hover:border-emerald-200 dark:hover:border-emerald-800 hover:text-emerald-600 dark:hover:text-emerald-400 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
                                            onClick={handleImportV1}
                                            disabled={status === 'loading' || status === 'success'}
                                        >
                                            <FileClock className="w-4 h-4" />
                                            {t('accounts.add.import.btn_v1')}
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>

                        <div className="flex gap-3 w-full mt-6">
                            <button
                                className="flex-1 px-4 py-2.5 bg-gray-100 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl hover:bg-gray-200 dark:hover:bg-base-300 transition-colors focus:outline-none focus:ring-2 focus:ring-200 dark:focus:ring-base-300"
                                onClick={async () => {
                                    if (status === 'loading' && activeTab === 'oauth') {
                                        await cancelOAuthLogin();
                                    }
                                    setIsOpen(false);
                                }}
                                disabled={status === 'success'} // Only disable on success, allow cancel on loading
                            >
                                {t('accounts.add.btn_cancel')}
                            </button>
                            {activeTab === 'token' && (
                                <button
                                    className="flex-1 px-4 py-2.5 text-white font-medium rounded-xl shadow-md transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 bg-blue-500 hover:bg-blue-600 focus:ring-blue-500 shadow-blue-100 dark:shadow-blue-900/30 flex justify-center items-center gap-2"
                                    onClick={handleSubmit}
                                    disabled={status === 'loading' || status === 'success'}
                                >
                                    {status === 'loading' ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                                    {t('accounts.add.btn_confirm')}
                                </button>
                            )}
                            {activeTab === 'manual' && provider === 'kiro' && (
                                <button
                                    className="flex-1 px-4 py-2.5 text-white font-medium rounded-xl shadow-md transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 bg-purple-500 hover:bg-purple-600 focus:ring-purple-500 shadow-purple-100 dark:shadow-purple-900/30 flex justify-center items-center gap-2"
                                    onClick={handleSubmitManualTokens}
                                    disabled={status === 'loading' || status === 'success' || !manualAccessToken.trim() || !manualRefreshToken.trim()}
                                >
                                    {status === 'loading' ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                                    Add Kiro Account
                                </button>
                            )}
                        </div>
                    </div>
                    <div className="modal-backdrop bg-black/40 backdrop-blur-sm fixed inset-0 z-[-1]" onClick={() => setIsOpen(false)}></div>
                </div>,
                document.body
            )}
        </>
    );
}

export default AddAccountDialog;
