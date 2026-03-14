// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'paginator-rs',
			logo: {
				light: './src/assets/logo-light.svg',
				dark: './src/assets/logo-dark.svg',
				replacesTitle: false,
			},
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/maulanasdqn/paginator-rs' },
			],
			customCss: ['./src/styles/custom.css'],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: '' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quick Start', slug: 'getting-started/quick-start' },
					],
				},
				{
					label: 'Core Concepts',
					items: [
						{ label: 'Builder Pattern', slug: 'core-concepts/builder-pattern' },
						{ label: 'Filtering', slug: 'core-concepts/filtering' },
						{ label: 'Sorting', slug: 'core-concepts/sorting' },
						{ label: 'Search', slug: 'core-concepts/search' },
						{ label: 'Cursor Pagination', slug: 'core-concepts/cursor-pagination' },
						{ label: 'Response Format', slug: 'core-concepts/response-format' },
					],
				},
				{
					label: 'Database Integrations',
					items: [
						{ label: 'SQLx', slug: 'database-integrations/sqlx' },
						{ label: 'SeaORM', slug: 'database-integrations/sea-orm' },
						{ label: 'SurrealDB', slug: 'database-integrations/surrealdb' },
					],
				},
				{
					label: 'Web Frameworks',
					items: [
						{ label: 'Axum', slug: 'web-frameworks/axum' },
						{ label: 'Rocket', slug: 'web-frameworks/rocket' },
						{ label: 'Actix-web', slug: 'web-frameworks/actix' },
					],
				},
				{
					label: 'Advanced',
					items: [
						{ label: 'Error Handling', slug: 'advanced/error-handling' },
						{ label: 'Performance', slug: 'advanced/performance' },
						{ label: 'Security', slug: 'advanced/security' },
						{ label: 'Query Parameters', slug: 'advanced/query-parameters' },
					],
				},
			],
		}),
	],
});
