#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use openvr_sys::*;

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]

pub enum EventType {
    None = EVREventType_VREvent_None,
    TrackedDeviceActivated = EVREventType_VREvent_TrackedDeviceActivated,
    TrackedDeviceDeactivated = EVREventType_VREvent_TrackedDeviceDeactivated,
    TrackedDeviceUpdated = EVREventType_VREvent_TrackedDeviceUpdated,
    TrackedDeviceUserInteractionStarted = EVREventType_VREvent_TrackedDeviceUserInteractionStarted,
    TrackedDeviceUserInteractionEnded = EVREventType_VREvent_TrackedDeviceUserInteractionEnded,
    IpdChanged = EVREventType_VREvent_IpdChanged,
    EnterStandbyMode = EVREventType_VREvent_EnterStandbyMode,
    LeaveStandbyMode = EVREventType_VREvent_LeaveStandbyMode,
    TrackedDeviceRoleChanged = EVREventType_VREvent_TrackedDeviceRoleChanged,
    WatchdogWakeUpRequested = EVREventType_VREvent_WatchdogWakeUpRequested,
    LensDistortionChanged = EVREventType_VREvent_LensDistortionChanged,
    PropertyChanged = EVREventType_VREvent_PropertyChanged,
    WirelessDisconnect = EVREventType_VREvent_WirelessDisconnect,
    WirelessReconnect = EVREventType_VREvent_WirelessReconnect,
    ButtonPress = EVREventType_VREvent_ButtonPress,
    ButtonUnpress = EVREventType_VREvent_ButtonUnpress,
    ButtonTouch = EVREventType_VREvent_ButtonTouch,
    ButtonUntouch = EVREventType_VREvent_ButtonUntouch,
    DualAnalog_Press = EVREventType_VREvent_DualAnalog_Press,
    DualAnalog_Unpress = EVREventType_VREvent_DualAnalog_Unpress,
    DualAnalog_Touch = EVREventType_VREvent_DualAnalog_Touch,
    DualAnalog_Untouch = EVREventType_VREvent_DualAnalog_Untouch,
    DualAnalog_Move = EVREventType_VREvent_DualAnalog_Move,
    DualAnalog_ModeSwitch1 = EVREventType_VREvent_DualAnalog_ModeSwitch1,
    DualAnalog_ModeSwitch2 = EVREventType_VREvent_DualAnalog_ModeSwitch2,
    DualAnalog_Cancel = EVREventType_VREvent_DualAnalog_Cancel,
    MouseMove = EVREventType_VREvent_MouseMove,
    MouseButtonDown = EVREventType_VREvent_MouseButtonDown,
    MouseButtonUp = EVREventType_VREvent_MouseButtonUp,
    FocusEnter = EVREventType_VREvent_FocusEnter,
    FocusLeave = EVREventType_VREvent_FocusLeave,
    Scroll = EVREventType_VREvent_Scroll,
    TouchPadMove = EVREventType_VREvent_TouchPadMove,
    OverlayFocusChanged = EVREventType_VREvent_OverlayFocusChanged,
    ReloadOverlays = EVREventType_VREvent_ReloadOverlays,
    InputFocusCaptured = EVREventType_VREvent_InputFocusCaptured,
    InputFocusReleased = EVREventType_VREvent_InputFocusReleased,
    SceneFocusLost = EVREventType_VREvent_SceneFocusLost,
    SceneFocusGained = EVREventType_VREvent_SceneFocusGained,
    SceneApplicationChanged = EVREventType_VREvent_SceneApplicationChanged,
    SceneFocusChanged = EVREventType_VREvent_SceneFocusChanged,
    InputFocusChanged = EVREventType_VREvent_InputFocusChanged,
    SceneApplicationSecondaryRenderingStarted =
        EVREventType_VREvent_SceneApplicationSecondaryRenderingStarted,
    SceneApplicationUsingWrongGraphicsAdapter =
        EVREventType_VREvent_SceneApplicationUsingWrongGraphicsAdapter,
    ActionBindingReloaded = EVREventType_VREvent_ActionBindingReloaded,
    HideRenderModels = EVREventType_VREvent_HideRenderModels,
    ShowRenderModels = EVREventType_VREvent_ShowRenderModels,
    ConsoleOpened = EVREventType_VREvent_ConsoleOpened,
    ConsoleClosed = EVREventType_VREvent_ConsoleClosed,
    OverlayShown = EVREventType_VREvent_OverlayShown,
    OverlayHidden = EVREventType_VREvent_OverlayHidden,
    DashboardActivated = EVREventType_VREvent_DashboardActivated,
    DashboardDeactivated = EVREventType_VREvent_DashboardDeactivated,
    DashboardThumbSelected = EVREventType_VREvent_DashboardThumbSelected,
    DashboardRequested = EVREventType_VREvent_DashboardRequested,
    ResetDashboard = EVREventType_VREvent_ResetDashboard,
    RenderToast = EVREventType_VREvent_RenderToast,
    ImageLoaded = EVREventType_VREvent_ImageLoaded,
    ShowKeyboard = EVREventType_VREvent_ShowKeyboard,
    HideKeyboard = EVREventType_VREvent_HideKeyboard,
    OverlayGamepadFocusGained = EVREventType_VREvent_OverlayGamepadFocusGained,
    OverlayGamepadFocusLost = EVREventType_VREvent_OverlayGamepadFocusLost,
    OverlaySharedTextureChanged = EVREventType_VREvent_OverlaySharedTextureChanged,
    ScreenshotTriggered = EVREventType_VREvent_ScreenshotTriggered,
    ImageFailed = EVREventType_VREvent_ImageFailed,
    DashboardOverlayCreated = EVREventType_VREvent_DashboardOverlayCreated,
    SwitchGamepadFocus = EVREventType_VREvent_SwitchGamepadFocus,
    RequestScreenshot = EVREventType_VREvent_RequestScreenshot,
    ScreenshotTaken = EVREventType_VREvent_ScreenshotTaken,
    ScreenshotFailed = EVREventType_VREvent_ScreenshotFailed,
    SubmitScreenshotToDashboard = EVREventType_VREvent_SubmitScreenshotToDashboard,
    ScreenshotProgressToDashboard = EVREventType_VREvent_ScreenshotProgressToDashboard,
    PrimaryDashboardDeviceChanged = EVREventType_VREvent_PrimaryDashboardDeviceChanged,
    RoomViewShown = EVREventType_VREvent_RoomViewShown,
    RoomViewHidden = EVREventType_VREvent_RoomViewHidden,
    ShowUI = EVREventType_VREvent_ShowUI,
    Notification_Shown = EVREventType_VREvent_Notification_Shown,
    Notification_Hidden = EVREventType_VREvent_Notification_Hidden,
    Notification_BeginInteraction = EVREventType_VREvent_Notification_BeginInteraction,
    Notification_Destroyed = EVREventType_VREvent_Notification_Destroyed,
    Quit = EVREventType_VREvent_Quit,
    ProcessQuit = EVREventType_VREvent_ProcessQuit,
    QuitAborted_UserPrompt = EVREventType_VREvent_QuitAborted_UserPrompt,
    QuitAcknowledged = EVREventType_VREvent_QuitAcknowledged,
    DriverRequestedQuit = EVREventType_VREvent_DriverRequestedQuit,
    ChaperoneDataHasChanged = EVREventType_VREvent_ChaperoneDataHasChanged,
    ChaperoneUniverseHasChanged = EVREventType_VREvent_ChaperoneUniverseHasChanged,
    ChaperoneTempDataHasChanged = EVREventType_VREvent_ChaperoneTempDataHasChanged,
    ChaperoneSettingsHaveChanged = EVREventType_VREvent_ChaperoneSettingsHaveChanged,
    SeatedZeroPoseReset = EVREventType_VREvent_SeatedZeroPoseReset,
    ChaperoneFlushCache = EVREventType_VREvent_ChaperoneFlushCache,
    AudioSettingsHaveChanged = EVREventType_VREvent_AudioSettingsHaveChanged,
    BackgroundSettingHasChanged = EVREventType_VREvent_BackgroundSettingHasChanged,
    CameraSettingsHaveChanged = EVREventType_VREvent_CameraSettingsHaveChanged,
    ReprojectionSettingHasChanged = EVREventType_VREvent_ReprojectionSettingHasChanged,
    ModelSkinSettingsHaveChanged = EVREventType_VREvent_ModelSkinSettingsHaveChanged,
    EnvironmentSettingsHaveChanged = EVREventType_VREvent_EnvironmentSettingsHaveChanged,
    PowerSettingsHaveChanged = EVREventType_VREvent_PowerSettingsHaveChanged,
    EnableHomeAppSettingsHaveChanged = EVREventType_VREvent_EnableHomeAppSettingsHaveChanged,
    SteamVRSectionSettingChanged = EVREventType_VREvent_SteamVRSectionSettingChanged,
    LighthouseSectionSettingChanged = EVREventType_VREvent_LighthouseSectionSettingChanged,
    NullSectionSettingChanged = EVREventType_VREvent_NullSectionSettingChanged,
    UserInterfaceSectionSettingChanged = EVREventType_VREvent_UserInterfaceSectionSettingChanged,
    NotificationsSectionSettingChanged = EVREventType_VREvent_NotificationsSectionSettingChanged,
    KeyboardSectionSettingChanged = EVREventType_VREvent_KeyboardSectionSettingChanged,
    PerfSectionSettingChanged = EVREventType_VREvent_PerfSectionSettingChanged,
    DashboardSectionSettingChanged = EVREventType_VREvent_DashboardSectionSettingChanged,
    WebInterfaceSectionSettingChanged = EVREventType_VREvent_WebInterfaceSectionSettingChanged,
    TrackersSectionSettingChanged = EVREventType_VREvent_TrackersSectionSettingChanged,
    LastKnownSectionSettingChanged = EVREventType_VREvent_LastKnownSectionSettingChanged,
    StatusUpdate = EVREventType_VREvent_StatusUpdate,
    WebInterface_InstallDriverCompleted = EVREventType_VREvent_WebInterface_InstallDriverCompleted,
    MCImageUpdated = EVREventType_VREvent_MCImageUpdated,
    FirmwareUpdateStarted = EVREventType_VREvent_FirmwareUpdateStarted,
    FirmwareUpdateFinished = EVREventType_VREvent_FirmwareUpdateFinished,
    KeyboardClosed = EVREventType_VREvent_KeyboardClosed,
    KeyboardCharInput = EVREventType_VREvent_KeyboardCharInput,
    KeyboardDone = EVREventType_VREvent_KeyboardDone,
    ApplicationTransitionStarted = EVREventType_VREvent_ApplicationTransitionStarted,
    ApplicationTransitionAborted = EVREventType_VREvent_ApplicationTransitionAborted,
    ApplicationTransitionNewAppStarted = EVREventType_VREvent_ApplicationTransitionNewAppStarted,
    ApplicationListUpdated = EVREventType_VREvent_ApplicationListUpdated,
    ApplicationMimeTypeLoad = EVREventType_VREvent_ApplicationMimeTypeLoad,
    ApplicationTransitionNewAppLaunchComplete =
        EVREventType_VREvent_ApplicationTransitionNewAppLaunchComplete,
    ProcessConnected = EVREventType_VREvent_ProcessConnected,
    ProcessDisconnected = EVREventType_VREvent_ProcessDisconnected,
    Compositor_MirrorWindowShown = EVREventType_VREvent_Compositor_MirrorWindowShown,
    Compositor_MirrorWindowHidden = EVREventType_VREvent_Compositor_MirrorWindowHidden,
    Compositor_ChaperoneBoundsShown = EVREventType_VREvent_Compositor_ChaperoneBoundsShown,
    Compositor_ChaperoneBoundsHidden = EVREventType_VREvent_Compositor_ChaperoneBoundsHidden,
    TrackedCamera_StartVideoStream = EVREventType_VREvent_TrackedCamera_StartVideoStream,
    TrackedCamera_StopVideoStream = EVREventType_VREvent_TrackedCamera_StopVideoStream,
    TrackedCamera_PauseVideoStream = EVREventType_VREvent_TrackedCamera_PauseVideoStream,
    TrackedCamera_ResumeVideoStream = EVREventType_VREvent_TrackedCamera_ResumeVideoStream,
    TrackedCamera_EditingSurface = EVREventType_VREvent_TrackedCamera_EditingSurface,
    PerformanceTest_EnableCapture = EVREventType_VREvent_PerformanceTest_EnableCapture,
    PerformanceTest_DisableCapture = EVREventType_VREvent_PerformanceTest_DisableCapture,
    PerformanceTest_FidelityLevel = EVREventType_VREvent_PerformanceTest_FidelityLevel,
    MessageOverlay_Closed = EVREventType_VREvent_MessageOverlay_Closed,
    MessageOverlayCloseRequested = EVREventType_VREvent_MessageOverlayCloseRequested,
    Input_HapticVibration = EVREventType_VREvent_Input_HapticVibration,
    Input_BindingLoadFailed = EVREventType_VREvent_Input_BindingLoadFailed,
    Input_BindingLoadSuccessful = EVREventType_VREvent_Input_BindingLoadSuccessful,
    Input_ActionManifestReloaded = EVREventType_VREvent_Input_ActionManifestReloaded,
    Input_ActionManifestLoadFailed = EVREventType_VREvent_Input_ActionManifestLoadFailed,
    Input_ProgressUpdate = EVREventType_VREvent_Input_ProgressUpdate,
    Input_TrackerActivated = EVREventType_VREvent_Input_TrackerActivated,
    SpatialAnchors_PoseUpdated = EVREventType_VREvent_SpatialAnchors_PoseUpdated,
    SpatialAnchors_DescriptorUpdated = EVREventType_VREvent_SpatialAnchors_DescriptorUpdated,
    SpatialAnchors_RequestPoseUpdate = EVREventType_VREvent_SpatialAnchors_RequestPoseUpdate,
    SpatialAnchors_RequestDescriptorUpdate =
        EVREventType_VREvent_SpatialAnchors_RequestDescriptorUpdate,
}

impl EventType {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            EVREventType_VREvent_None => EventType::None,
            EVREventType_VREvent_TrackedDeviceActivated => EventType::TrackedDeviceActivated,
            EVREventType_VREvent_TrackedDeviceDeactivated => EventType::TrackedDeviceDeactivated,
            EVREventType_VREvent_TrackedDeviceUpdated => EventType::TrackedDeviceUpdated,
            EVREventType_VREvent_TrackedDeviceUserInteractionStarted => {
                EventType::TrackedDeviceUserInteractionStarted
            }
            EVREventType_VREvent_TrackedDeviceUserInteractionEnded => {
                EventType::TrackedDeviceUserInteractionEnded
            }
            EVREventType_VREvent_IpdChanged => EventType::IpdChanged,
            EVREventType_VREvent_EnterStandbyMode => EventType::EnterStandbyMode,
            EVREventType_VREvent_LeaveStandbyMode => EventType::LeaveStandbyMode,
            EVREventType_VREvent_TrackedDeviceRoleChanged => EventType::TrackedDeviceRoleChanged,
            EVREventType_VREvent_WatchdogWakeUpRequested => EventType::WatchdogWakeUpRequested,
            EVREventType_VREvent_LensDistortionChanged => EventType::LensDistortionChanged,
            EVREventType_VREvent_PropertyChanged => EventType::PropertyChanged,
            EVREventType_VREvent_WirelessDisconnect => EventType::WirelessDisconnect,
            EVREventType_VREvent_WirelessReconnect => EventType::WirelessReconnect,
            EVREventType_VREvent_ButtonPress => EventType::ButtonPress,
            EVREventType_VREvent_ButtonUnpress => EventType::ButtonUnpress,
            EVREventType_VREvent_ButtonTouch => EventType::ButtonTouch,
            EVREventType_VREvent_ButtonUntouch => EventType::ButtonUntouch,
            EVREventType_VREvent_DualAnalog_Press => EventType::DualAnalog_Press,
            EVREventType_VREvent_DualAnalog_Unpress => EventType::DualAnalog_Unpress,
            EVREventType_VREvent_DualAnalog_Touch => EventType::DualAnalog_Touch,
            EVREventType_VREvent_DualAnalog_Untouch => EventType::DualAnalog_Untouch,
            EVREventType_VREvent_DualAnalog_Move => EventType::DualAnalog_Move,
            EVREventType_VREvent_DualAnalog_ModeSwitch1 => EventType::DualAnalog_ModeSwitch1,
            EVREventType_VREvent_DualAnalog_ModeSwitch2 => EventType::DualAnalog_ModeSwitch2,
            EVREventType_VREvent_DualAnalog_Cancel => EventType::DualAnalog_Cancel,
            EVREventType_VREvent_MouseMove => EventType::MouseMove,
            EVREventType_VREvent_MouseButtonDown => EventType::MouseButtonDown,
            EVREventType_VREvent_MouseButtonUp => EventType::MouseButtonUp,
            EVREventType_VREvent_FocusEnter => EventType::FocusEnter,
            EVREventType_VREvent_FocusLeave => EventType::FocusLeave,
            EVREventType_VREvent_Scroll => EventType::Scroll,
            EVREventType_VREvent_TouchPadMove => EventType::TouchPadMove,
            EVREventType_VREvent_OverlayFocusChanged => EventType::OverlayFocusChanged,
            EVREventType_VREvent_ReloadOverlays => EventType::ReloadOverlays,
            EVREventType_VREvent_InputFocusCaptured => EventType::InputFocusCaptured,
            EVREventType_VREvent_InputFocusReleased => EventType::InputFocusReleased,
            EVREventType_VREvent_SceneFocusLost => EventType::SceneFocusLost,
            EVREventType_VREvent_SceneFocusGained => EventType::SceneFocusGained,
            EVREventType_VREvent_SceneApplicationChanged => EventType::SceneApplicationChanged,
            EVREventType_VREvent_SceneFocusChanged => EventType::SceneFocusChanged,
            EVREventType_VREvent_InputFocusChanged => EventType::InputFocusChanged,
            EVREventType_VREvent_SceneApplicationSecondaryRenderingStarted => {
                EventType::SceneApplicationSecondaryRenderingStarted
            }
            EVREventType_VREvent_SceneApplicationUsingWrongGraphicsAdapter => {
                EventType::SceneApplicationUsingWrongGraphicsAdapter
            }
            EVREventType_VREvent_ActionBindingReloaded => EventType::ActionBindingReloaded,
            EVREventType_VREvent_HideRenderModels => EventType::HideRenderModels,
            EVREventType_VREvent_ShowRenderModels => EventType::ShowRenderModels,
            EVREventType_VREvent_ConsoleOpened => EventType::ConsoleOpened,
            EVREventType_VREvent_ConsoleClosed => EventType::ConsoleClosed,
            EVREventType_VREvent_OverlayShown => EventType::OverlayShown,
            EVREventType_VREvent_OverlayHidden => EventType::OverlayHidden,
            EVREventType_VREvent_DashboardActivated => EventType::DashboardActivated,
            EVREventType_VREvent_DashboardDeactivated => EventType::DashboardDeactivated,
            EVREventType_VREvent_DashboardThumbSelected => EventType::DashboardThumbSelected,
            EVREventType_VREvent_DashboardRequested => EventType::DashboardRequested,
            EVREventType_VREvent_ResetDashboard => EventType::ResetDashboard,
            EVREventType_VREvent_RenderToast => EventType::RenderToast,
            EVREventType_VREvent_ImageLoaded => EventType::ImageLoaded,
            EVREventType_VREvent_ShowKeyboard => EventType::ShowKeyboard,
            EVREventType_VREvent_HideKeyboard => EventType::HideKeyboard,
            EVREventType_VREvent_OverlayGamepadFocusGained => EventType::OverlayGamepadFocusGained,
            EVREventType_VREvent_OverlayGamepadFocusLost => EventType::OverlayGamepadFocusLost,
            EVREventType_VREvent_OverlaySharedTextureChanged => {
                EventType::OverlaySharedTextureChanged
            }
            EVREventType_VREvent_ScreenshotTriggered => EventType::ScreenshotTriggered,
            EVREventType_VREvent_ImageFailed => EventType::ImageFailed,
            EVREventType_VREvent_DashboardOverlayCreated => EventType::DashboardOverlayCreated,
            EVREventType_VREvent_SwitchGamepadFocus => EventType::SwitchGamepadFocus,
            EVREventType_VREvent_RequestScreenshot => EventType::RequestScreenshot,
            EVREventType_VREvent_ScreenshotTaken => EventType::ScreenshotTaken,
            EVREventType_VREvent_ScreenshotFailed => EventType::ScreenshotFailed,
            EVREventType_VREvent_SubmitScreenshotToDashboard => {
                EventType::SubmitScreenshotToDashboard
            }
            EVREventType_VREvent_ScreenshotProgressToDashboard => {
                EventType::ScreenshotProgressToDashboard
            }
            EVREventType_VREvent_PrimaryDashboardDeviceChanged => {
                EventType::PrimaryDashboardDeviceChanged
            }
            EVREventType_VREvent_RoomViewShown => EventType::RoomViewShown,
            EVREventType_VREvent_RoomViewHidden => EventType::RoomViewHidden,
            EVREventType_VREvent_ShowUI => EventType::ShowUI,
            EVREventType_VREvent_Notification_Shown => EventType::Notification_Shown,
            EVREventType_VREvent_Notification_Hidden => EventType::Notification_Hidden,
            EVREventType_VREvent_Notification_BeginInteraction => {
                EventType::Notification_BeginInteraction
            }
            EVREventType_VREvent_Notification_Destroyed => EventType::Notification_Destroyed,
            EVREventType_VREvent_Quit => EventType::Quit,
            EVREventType_VREvent_ProcessQuit => EventType::ProcessQuit,
            EVREventType_VREvent_QuitAborted_UserPrompt => EventType::QuitAborted_UserPrompt,
            EVREventType_VREvent_QuitAcknowledged => EventType::QuitAcknowledged,
            EVREventType_VREvent_DriverRequestedQuit => EventType::DriverRequestedQuit,
            EVREventType_VREvent_ChaperoneDataHasChanged => EventType::ChaperoneDataHasChanged,
            EVREventType_VREvent_ChaperoneUniverseHasChanged => {
                EventType::ChaperoneUniverseHasChanged
            }
            EVREventType_VREvent_ChaperoneTempDataHasChanged => {
                EventType::ChaperoneTempDataHasChanged
            }
            EVREventType_VREvent_ChaperoneSettingsHaveChanged => {
                EventType::ChaperoneSettingsHaveChanged
            }
            EVREventType_VREvent_SeatedZeroPoseReset => EventType::SeatedZeroPoseReset,
            EVREventType_VREvent_ChaperoneFlushCache => EventType::ChaperoneFlushCache,
            EVREventType_VREvent_AudioSettingsHaveChanged => EventType::AudioSettingsHaveChanged,
            EVREventType_VREvent_BackgroundSettingHasChanged => {
                EventType::BackgroundSettingHasChanged
            }
            EVREventType_VREvent_CameraSettingsHaveChanged => EventType::CameraSettingsHaveChanged,
            EVREventType_VREvent_ReprojectionSettingHasChanged => {
                EventType::ReprojectionSettingHasChanged
            }
            EVREventType_VREvent_ModelSkinSettingsHaveChanged => {
                EventType::ModelSkinSettingsHaveChanged
            }
            EVREventType_VREvent_EnvironmentSettingsHaveChanged => {
                EventType::EnvironmentSettingsHaveChanged
            }
            EVREventType_VREvent_PowerSettingsHaveChanged => EventType::PowerSettingsHaveChanged,
            EVREventType_VREvent_EnableHomeAppSettingsHaveChanged => {
                EventType::EnableHomeAppSettingsHaveChanged
            }
            EVREventType_VREvent_SteamVRSectionSettingChanged => {
                EventType::SteamVRSectionSettingChanged
            }
            EVREventType_VREvent_LighthouseSectionSettingChanged => {
                EventType::LighthouseSectionSettingChanged
            }
            EVREventType_VREvent_NullSectionSettingChanged => EventType::NullSectionSettingChanged,
            EVREventType_VREvent_UserInterfaceSectionSettingChanged => {
                EventType::UserInterfaceSectionSettingChanged
            }
            EVREventType_VREvent_NotificationsSectionSettingChanged => {
                EventType::NotificationsSectionSettingChanged
            }
            EVREventType_VREvent_KeyboardSectionSettingChanged => {
                EventType::KeyboardSectionSettingChanged
            }
            EVREventType_VREvent_PerfSectionSettingChanged => EventType::PerfSectionSettingChanged,
            EVREventType_VREvent_DashboardSectionSettingChanged => {
                EventType::DashboardSectionSettingChanged
            }
            EVREventType_VREvent_WebInterfaceSectionSettingChanged => {
                EventType::WebInterfaceSectionSettingChanged
            }
            EVREventType_VREvent_TrackersSectionSettingChanged => {
                EventType::TrackersSectionSettingChanged
            }
            EVREventType_VREvent_LastKnownSectionSettingChanged => {
                EventType::LastKnownSectionSettingChanged
            }
            EVREventType_VREvent_StatusUpdate => EventType::StatusUpdate,
            EVREventType_VREvent_WebInterface_InstallDriverCompleted => {
                EventType::WebInterface_InstallDriverCompleted
            }
            EVREventType_VREvent_MCImageUpdated => EventType::MCImageUpdated,
            EVREventType_VREvent_FirmwareUpdateStarted => EventType::FirmwareUpdateStarted,
            EVREventType_VREvent_FirmwareUpdateFinished => EventType::FirmwareUpdateFinished,
            EVREventType_VREvent_KeyboardClosed => EventType::KeyboardClosed,
            EVREventType_VREvent_KeyboardCharInput => EventType::KeyboardCharInput,
            EVREventType_VREvent_KeyboardDone => EventType::KeyboardDone,
            EVREventType_VREvent_ApplicationTransitionStarted => {
                EventType::ApplicationTransitionStarted
            }
            EVREventType_VREvent_ApplicationTransitionAborted => {
                EventType::ApplicationTransitionAborted
            }
            EVREventType_VREvent_ApplicationTransitionNewAppStarted => {
                EventType::ApplicationTransitionNewAppStarted
            }
            EVREventType_VREvent_ApplicationListUpdated => EventType::ApplicationListUpdated,
            EVREventType_VREvent_ApplicationMimeTypeLoad => EventType::ApplicationMimeTypeLoad,
            EVREventType_VREvent_ApplicationTransitionNewAppLaunchComplete => {
                EventType::ApplicationTransitionNewAppLaunchComplete
            }
            EVREventType_VREvent_ProcessConnected => EventType::ProcessConnected,
            EVREventType_VREvent_ProcessDisconnected => EventType::ProcessDisconnected,
            EVREventType_VREvent_Compositor_MirrorWindowShown => {
                EventType::Compositor_MirrorWindowShown
            }
            EVREventType_VREvent_Compositor_MirrorWindowHidden => {
                EventType::Compositor_MirrorWindowHidden
            }
            EVREventType_VREvent_Compositor_ChaperoneBoundsShown => {
                EventType::Compositor_ChaperoneBoundsShown
            }
            EVREventType_VREvent_Compositor_ChaperoneBoundsHidden => {
                EventType::Compositor_ChaperoneBoundsHidden
            }
            EVREventType_VREvent_TrackedCamera_StartVideoStream => {
                EventType::TrackedCamera_StartVideoStream
            }
            EVREventType_VREvent_TrackedCamera_StopVideoStream => {
                EventType::TrackedCamera_StopVideoStream
            }
            EVREventType_VREvent_TrackedCamera_PauseVideoStream => {
                EventType::TrackedCamera_PauseVideoStream
            }
            EVREventType_VREvent_TrackedCamera_ResumeVideoStream => {
                EventType::TrackedCamera_ResumeVideoStream
            }
            EVREventType_VREvent_TrackedCamera_EditingSurface => {
                EventType::TrackedCamera_EditingSurface
            }
            EVREventType_VREvent_PerformanceTest_EnableCapture => {
                EventType::PerformanceTest_EnableCapture
            }
            EVREventType_VREvent_PerformanceTest_DisableCapture => {
                EventType::PerformanceTest_DisableCapture
            }
            EVREventType_VREvent_PerformanceTest_FidelityLevel => {
                EventType::PerformanceTest_FidelityLevel
            }
            EVREventType_VREvent_MessageOverlay_Closed => EventType::MessageOverlay_Closed,
            EVREventType_VREvent_MessageOverlayCloseRequested => {
                EventType::MessageOverlayCloseRequested
            }
            EVREventType_VREvent_Input_HapticVibration => EventType::Input_HapticVibration,
            EVREventType_VREvent_Input_BindingLoadFailed => EventType::Input_BindingLoadFailed,
            EVREventType_VREvent_Input_BindingLoadSuccessful => {
                EventType::Input_BindingLoadSuccessful
            }
            EVREventType_VREvent_Input_ActionManifestReloaded => {
                EventType::Input_ActionManifestReloaded
            }
            EVREventType_VREvent_Input_ActionManifestLoadFailed => {
                EventType::Input_ActionManifestLoadFailed
            }
            EVREventType_VREvent_Input_ProgressUpdate => EventType::Input_ProgressUpdate,
            EVREventType_VREvent_Input_TrackerActivated => EventType::Input_TrackerActivated,
            EVREventType_VREvent_SpatialAnchors_PoseUpdated => {
                EventType::SpatialAnchors_PoseUpdated
            }
            EVREventType_VREvent_SpatialAnchors_DescriptorUpdated => {
                EventType::SpatialAnchors_DescriptorUpdated
            }
            EVREventType_VREvent_SpatialAnchors_RequestPoseUpdate => {
                EventType::SpatialAnchors_RequestPoseUpdate
            }
            EVREventType_VREvent_SpatialAnchors_RequestDescriptorUpdate => {
                EventType::SpatialAnchors_RequestDescriptorUpdate
            }
            _ => return None,
        })
    }
}
