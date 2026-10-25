import { createBrowserRouter, Navigate } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { ProtectedRoute } from "@/components/layout/ProtectedRoute";
import { AchievementDetailPage } from "@/features/achievements/AchievementDetailPage";
import { AchievementEditorPage } from "@/features/achievements/AchievementEditorPage";
import { AchievementListPage } from "@/features/achievements/AchievementListPage";
import { LoginPage } from "@/features/auth/LoginPage";
import { DashboardPage } from "@/features/dashboard/DashboardPage";
import { FilesPage } from "@/features/files/FilesPage";
import { MyProfilePage } from "@/features/teachers/MyProfilePage";
import { ErrorPage } from "@/pages/ErrorPage";
import { appPaths } from "@/routes/routePaths";

export const router = createBrowserRouter([
	{ path: appPaths.login, element: <LoginPage />, errorElement: <ErrorPage /> },
	{
		element: <ProtectedRoute />,
		errorElement: <ErrorPage />,
		children: [
			{
				element: <AppShell />,
				children: [
					{ path: "/", element: <Navigate to={appPaths.dashboard} replace /> },
					{ path: appPaths.dashboard, element: <DashboardPage /> },
					{ path: appPaths.profile, element: <MyProfilePage /> },
					{ path: "/achievements", element: <AchievementListPage /> },
					{ path: "/achievements/new", element: <AchievementEditorPage /> },
					{ path: "/achievements/:id", element: <AchievementDetailPage /> },
					{
						path: "/achievements/:id/edit",
						element: <AchievementEditorPage />,
					},
					{ path: "/files", element: <FilesPage /> },
					{ path: "*", element: <ErrorPage /> },
				],
			},
		],
	},
]);
